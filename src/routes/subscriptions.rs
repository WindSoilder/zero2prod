use crate::{EmailClient, Request};

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
    add_subscriber(
        new_subscriber,
        &req.state().connection,
        &req.state().email_client,
        &req.state().base_url,
    )
    .await
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
    skip(pool, email_client),
    fields(request_id = %Uuid::new_v4())
)]
async fn add_subscriber(
    new_subscriber: NewSubscriber,
    pool: &PgPool,
    email_client: &EmailClient,
    base_url: &str,
) -> Result {
    if insert_subscriber(&new_subscriber, pool).await.is_err() {
        return Ok(Response::builder(StatusCode::InternalServerError).build());
    }
    if send_confirmation_email(email_client, new_subscriber, base_url)
        .await
        .is_err()
    {
        return Ok(Response::builder(StatusCode::InternalServerError).build());
    }

    Ok("".into())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
) -> std::result::Result<(), surf::Error> {
    // Send a (useless) email to the new subscriber.
    // We are ignoring email delivery errors for now.
    let confirmation_link = format!("{base_url}/subscriptions/confirm?subscription_token=mytoken");
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome",
            &format!(
                "Welcome to our newsletter!<br />\
            Clink <a href=\"{confirmation_link}\">here</a> to confirm your subscription.",
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {confirmation_link} to confirm your subscription.",
            )

        )
        .await
}

#[tracing::instrument(name = "Savning new subscriber details in the database")]
async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
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
