use crate::Request;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tide::Result;
use tide::{Response, StatusCode};
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
struct SubscribeBody {
    email: String,
    name: String,
}

pub async fn subscribe(mut req: Request) -> Result {
    let subscribe_body: SubscribeBody = req.body_form().await.map_err(|mut e| {
        e.set_status(400);
        e
    })?;
    add_subscriber(
        subscribe_body.name,
        subscribe_body.email,
        &req.state().connection,
    )
    .await
}

// WARN: can't use `name` as argument name for `add_subscriber`, or tracing will not show that argument.
// Because it already have a `name` field for Layer.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(pool),
    fields(request_id = %Uuid::new_v4())
)]
async fn add_subscriber(username: String, email: String, pool: &PgPool) -> Result {
    match insert_subscriber(username, email, pool).await {
        Ok(_) => Ok("".into()),
        Err(_) => Ok(Response::builder(StatusCode::InternalServerError).build()),
    }
}

#[tracing::instrument(name = "Savning new subscriber details in the database")]
async fn insert_subscriber(
    username: String,
    email: String,
    pool: &PgPool,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        email,
        username,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {e:?}");
        e
    })?;
    Ok(())
}
