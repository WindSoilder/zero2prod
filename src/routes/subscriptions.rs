use crate::Request;

use crate::domain::{NewSubscriber, SubscriberName};
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
    let name = match SubscriberName::parse(subscribe_body.name) {
        Ok(name) => name,
        // Return early if the name is invalid, with a 400
        Err(_) => return Ok(Response::builder(StatusCode::BadRequest).build()),
    };
    let new_subscriber = NewSubscriber {
        email: subscribe_body.email,
        name,
    };
    add_subscriber(&new_subscriber, &req.state().connection).await
}

// WARN: can't use `name` as argument name for `add_subscriber`, or tracing will not show that argument.
// Because it already have a `name` field for Layer.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(pool),
    fields(request_id = %Uuid::new_v4())
)]
async fn add_subscriber(new_subscriber: &NewSubscriber, pool: &PgPool) -> Result {
    match insert_subscriber(new_subscriber, pool).await {
        Ok(_) => Ok("".into()),
        Err(_) => Ok(Response::builder(StatusCode::InternalServerError).build()),
    }
}

#[tracing::instrument(name = "Savning new subscriber details in the database")]
async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
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
