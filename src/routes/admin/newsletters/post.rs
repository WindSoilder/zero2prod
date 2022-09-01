use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::utils::attach_flashed_message;
use crate::Request;
use anyhow::Context;
use sqlx::PgPool;
use tide::StatusCode;
use tide::{Redirect, Result};
#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    html_content: String,
    text_content: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

pub async fn publish_newsletter(mut req: Request) -> Result {
    let body: BodyData = req.body_form().await.map_err(|mut e| {
        e.set_status(StatusCode::BadRequest);
        e
    })?;
    let pool = &req.state().connection;
    let email_client = &req.state().email_client;
    publish_impl(pool, email_client, body).await?;

    let mut resp = Redirect::see_other("/admin/newsletters").into();
    let hmac_key = &req.state().hmac_secret;
    attach_flashed_message(
        &mut resp,
        hmac_key,
        "The newsletter issue has been published!".to_string(),
    );
    Ok(resp)
}

async fn publish_impl(pool: &PgPool, email_client: &EmailClient, body: BodyData) -> Result {
    let subscribers = get_confirmed_subscribers(pool).await?;
    for s in subscribers {
        match s {
            Ok(s) => {
                email_client
                    .send_email(
                        &s.email,
                        &body.title,
                        &body.html_content,
                        &body.text_content,
                    )
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
