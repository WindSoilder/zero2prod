use crate::Request;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
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
    let new_subscriber = match subscribe_body.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return Ok(Response::builder(StatusCode::BadRequest).build()),
    };
    add_subscriber(&new_subscriber, &req.state().connection).await
}

impl TryFrom<SubscribeBody> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscribeBody) -> std::result::Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
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
        new_subscriber.email.as_ref(),
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
