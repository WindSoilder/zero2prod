use std::fmt::Debug;

use crate::{EmailClient, Request};

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::{Postgres, Transaction};
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
    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Ok(Response::builder(StatusCode::InternalServerError).build()),
    };
    let subscriber_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return Ok(Response::builder(StatusCode::InternalServerError).build()),
    };
    let subscriber_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscriber_token).await?;

    if send_confirmation_email(email_client, new_subscriber, base_url, &subscriber_token)
        .await
        .is_err()
    {
        return Ok(Response::builder(StatusCode::InternalServerError).build());
    }
    if transaction.commit().await.is_err() {
        return Ok(Response::builder(StatusCode::InternalServerError).build());
    }

    Ok("".into())
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> std::result::Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> std::result::Result<(), surf::Error> {
    // Send a (useless) email to the new subscriber.
    // We are ignoring email delivery errors for now.
    let confirmation_link =
        format!("{base_url}/subscriptions/confirm?subscription_token={subscription_token}");
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
    transaction: &mut Transaction<'_, Postgres>,
) -> std::result::Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {e:?}");
        e
    })?;
    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[derive(Debug)]
pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "database error occured: {:?}", self.0)
    }
}

impl std::error::Error for StoreTokenError {}
