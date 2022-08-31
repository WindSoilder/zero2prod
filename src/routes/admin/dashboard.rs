use crate::login_middleware::UserId;
use crate::Request;
use anyhow::Context;
use sqlx::PgPool;
use tide::{Response, Result};
use uuid::Uuid;

pub async fn admin_dashboard(req: Request) -> Result {
    let pool = &req.state().connection;
    let user_id: &UserId = req
        .ext::<UserId>()
        .expect("request session not initialized, did you enable crate::login_middleware::RequiredLoginMiddleware?");
    let username = get_username(user_id.0, pool).await?;
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome {username}!</p>
    <p> Available actions: </p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
        <li>
            <form name="logoutForm" action="/admin/logout" method="post">
                <input type="submit" value="Logout">
            </form>
        </li>
        <li><a href="/admin/newsletters">Send a newsletter issue</a></li>
    </ol>
</body>
</html>"#
    );
    let mut resp: Response = body.into();
    resp.set_content_type("text/html; charset=utf-8");
    Ok(resp)
}

pub async fn get_username(
    user_id: Uuid,
    pool: &PgPool,
) -> std::result::Result<String, anyhow::Error> {
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
