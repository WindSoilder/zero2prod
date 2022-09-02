use crate::idempotency::IdempotencyKey;
use crate::idempotency::{save_response, try_processing, NextAction};
use crate::login_middleware::UserId;
use crate::routes::utils::attach_flashed_message;
use crate::Request;
use anyhow::Context;
use sqlx::{Postgres, Transaction};
use tide::{Redirect, Result};
use tide::{Response, StatusCode};
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    html_content: String,
    text_content: String,
    idempotency_key: String,
}

pub async fn publish_newsletter(mut req: Request) -> Result {
    let body: BodyData = req.body_form().await.map_err(|mut e| {
        e.set_status(StatusCode::BadRequest);
        e
    })?;
    let BodyData {
        title,
        html_content,
        text_content,
        idempotency_key,
    } = body;
    let idempotency_key: IdempotencyKey = match idempotency_key.try_into() {
        Ok(k) => k,
        Err(e) => {
            let mut resp = Response::new(StatusCode::BadRequest);
            resp.set_error(e);
            return Ok(resp);
        }
    };
    // return early if we have a saved response in the database.
    let user_id = req
        .ext::<UserId>()
        .expect("make sure you've load login middleware")
        .0;
    let pool = &req.state().connection;
    let mut transaction = match try_processing(&pool, &idempotency_key, user_id).await? {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(mut saved_response) => {
            let hmac_key = &req.state().hmac_secret;
            attach_flashed_message(
                &mut saved_response,
                hmac_key,
                "The newsletter issue has been published!".to_string(),
            );
            return Ok(saved_response);
        }
    };
    let issue_id = insert_newsletter_issue(&mut transaction, &title, &text_content, &html_content)
        .await
        .context("Failed to store newsletter issue deetails")?;
    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")?;
    let mut resp = Redirect::see_other("/admin/newsletters").into();
    let hmac_key = &req.state().hmac_secret;
    attach_flashed_message(
        &mut resp,
        hmac_key,
        "The newsletter issue has been published!".to_string(),
    );
    let resp = save_response(transaction, &idempotency_key, user_id, resp).await?;
    Ok(resp)
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> std::result::Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO newsletter_issues (
        newsletter_issue_id,
        title,
        text_content,
        html_content,
        published_at
    )
    VALUES ($1, $2, $3, $4, now())
    "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    )
    .execute(transaction)
    .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue(
            newsletter_issue_id,
            subscriber_email
        )
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(transaction)
    .await?;
    Ok(())
}
