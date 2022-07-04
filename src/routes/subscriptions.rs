use crate::Request;

use chrono::Utc;
use serde::{Deserialize, Serialize};
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
    .await
    {
        Ok(_) => Ok("".into()),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            Ok(Response::builder(StatusCode::InternalServerError).build())
        }
    }
}
