use crate::Request;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tide::Result;
use tide::{Response, StatusCode};
use tracing::Instrument;
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
    // generate a random unique identifier.
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %subscribe_body.email,
        subscriber_name = %subscribe_body.name,
    );
    let _request_span_guard = request_span.enter();
    // We do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments
    // in the query future lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the databse");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        subscribe_body.email,
        subscribe_body.name,
        Utc::now()
    )
    .execute(&req.state().connection)
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("request_id {request_id} - New subscriber details have been saved");
            Ok("".into())
        }
        Err(e) => {
            tracing::error!("request_id {request_id} - Failed to execute query: {e:?}");
            Ok(Response::builder(StatusCode::InternalServerError).build())
        }
    }
}
