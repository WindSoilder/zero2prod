use crate::Request;
use serde::Deserialize;
use sqlx::PgPool;
use tide::StatusCode;
use tide::{Response, Result};
use uuid::Uuid;

#[derive(Deserialize)]
struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(req))]
pub async fn confirm(req: Request) -> Result {
    let parameters: Parameters = req.query()?;
    let pool = &req.state().connection;
    let id = match get_subscriber_id_from_token(&pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(StatusCode::InternalServerError).build()),
    };
    match id {
        // Non-exists token!
        None => return Ok(Response::builder(StatusCode::Unauthorized).build()),
        Some(subscriber_id) => {
            if confirm_subscriber(pool, subscriber_id).await.is_err() {
                return Ok(Response::builder(StatusCode::InternalServerError).build());
            }
            Ok("".into())
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {e:?}");
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {e:?}");
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}
