use crate::routes::utils::get_flashed_message;
use crate::Request;
use http_types::Cookie;
use tide::{Response, Result};

pub async fn login_form(req: Request) -> Result {
    let error_html = get_flashed_message(&req);
    let body = format!(
        r#"<!DOCTYPE html>
        <html lang="en">

        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Login</title>
        </head>

        <body>
            {error_html}
            <form action="/login" method="post">
                <label>Username <input type="text" placeholder="Enter Username" name="username"> </label>
                <label>Password <input type="password" placeholder="Enter Password" name="password"> </label>
                <button type="submit">Login</button>
            </form>
        </body>

        </html>"#,
    );
    let mut resp: Response = body.into();
    resp.set_content_type("text/html; charset=utf-8");
    resp.remove_cookie(Cookie::named("_flash"));
    resp.remove_cookie(Cookie::named("tag"));
    Ok(resp)
}
