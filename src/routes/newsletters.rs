use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::Request;
use anyhow::Context;
use http_types::headers;
use secrecy::Secret;
use sqlx::PgPool;
use tide::Result;
use tide::StatusCode;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

pub async fn publish_newsletter(mut req: Request) -> Result {
    let _credentials = match basic_authentication(&req) {
        Ok(c) => c,
        Err(e) => {
            let mut response: tide::Response = tide::Error::new(StatusCode::Unauthorized, e).into();
            response.append_header(headers::WWW_AUTHENTICATE, r#"Basic realm="publish""#);
            return Ok(response);
        }
    };
    let body: BodyData = req.body_json().await.map_err(|mut e| {
        e.set_status(StatusCode::BadRequest);
        e
    })?;
    let pool = &req.state().connection;
    let email_client = &req.state().email_client;
    publish_impl(pool, email_client, body).await?;

    Ok("".into())
}

struct Credentials {
    username: String,
    password: Secret<String>,
}

fn basic_authentication(req: &Request) -> std::result::Result<Credentials, anyhow::Error> {
    // The header value, if present, must be a valid UTF8 string.
    let header_value = req
        .header(http_types::headers::AUTHORIZATION)
        .context("The 'Authorization' header was missing")?
        .as_str();
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::decode_config(base64encoded_segment, base64::STANDARD)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decode_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8")?;

    // Split into two segments, using ':' as delimitator
    let mut credentials = decode_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
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
                        &body.content.html,
                        &body.content.text,
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
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
