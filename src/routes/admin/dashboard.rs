use crate::session_state::TypedSession;
use crate::Request;
use anyhow::Context;
use sqlx::PgPool;
use tide::{Redirect, Response, Result};
use uuid::Uuid;

pub async fn admin_dashboard(req: Request) -> Result {
    let session = TypedSession::from_req(&req);
    let pool = &req.state().connection;
    let username = match session.get_user_id() {
        None => return Ok(Redirect::see_other("/login").into()),
        Some(user_id) => get_username(user_id, pool).await?,
    };
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome {username}!</p>
</body>
</html>"#
    );
    let mut resp: Response = body.into();
    resp.set_content_type("text/html; charset=utf-8");
    Ok(resp)
}

async fn get_username(user_id: Uuid, pool: &PgPool) -> std::result::Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username FROM users WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
