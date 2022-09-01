use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::idempotency::get_saved_response;
use crate::idempotency::IdempotencyKey;
use crate::login_middleware::UserId;
use crate::routes::utils::attach_flashed_message;
use crate::Request;
use anyhow::Context;
use sqlx::PgPool;
use tide::{Redirect, Result};
use tide::{Response, StatusCode};
#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    html_content: String,
    text_content: String,
    idempotency_key: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
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
    if let Some(saved_response) = get_saved_response(pool, &idempotency_key, user_id).await? {
        return Ok(saved_response);
    }
    let email_client = &req.state().email_client;

    publish_impl(pool, email_client, title, html_content, text_content).await?;

    let mut resp = Redirect::see_other("/admin/newsletters").into();
    let hmac_key = &req.state().hmac_secret;
    attach_flashed_message(
        &mut resp,
        hmac_key,
        "The newsletter issue has been published!".to_string(),
    );
    Ok(resp)
}

async fn publish_impl(
    pool: &PgPool,
    email_client: &EmailClient,
    title: String,
    html_content: String,
    text_content: String,
) -> Result {
    let subscribers = get_confirmed_subscribers(pool).await?;
    for s in subscribers {
        match s {
            Ok(s) => {
                email_client
                    .send_email(&s.email, &title, &html_content, &text_content)
                    .await
                    .map_err(|e| e.into_inner())
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", s.email.as_ref())
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber.  Their stored contact details are invalid"
                )
            }
        }
    }
    Ok("".into())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> std::result::Result<Vec<std::result::Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error>
{
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();
    Ok(confirmed_subscribers)
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
