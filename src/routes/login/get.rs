use crate::Request;
use hmac::{Hmac, Mac};
use http_types::Cookie;
use secrecy::ExposeSecret;
use tide::{Response, Result};

pub async fn login_form(_req: Request) -> Result {
    let error_html = get_error_message(_req).unwrap_or_else(|_| "".to_string());
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

/// get error message from given request cookie
///
/// Returns false if `tag` doesn't exists in request cookie, or `tag` verified failed.
fn get_error_message(req: Request) -> std::result::Result<String, anyhow::Error> {
    let error_message = match req.cookie("_flash") {
        None => "".to_string(),
        Some(cookie) => cookie.value().to_string(),
    };
    let secret = &req.state().hmac_secret;

    match req.cookie("tag") {
        None => Ok("".to_string()),
        Some(tag) => {
            let tag = hex::decode(tag.value())?;
            let msg = format!("_flash={error_message}");
            let mut mac =
                Hmac::<sha2::Sha256>::new_from_slice(secret.expose_secret().as_bytes()).unwrap();
            mac.update(msg.as_bytes());
            // add debug...
            let mac_bytes = mac.clone().finalize().into_bytes();
            println!("verified input bytes: {mac_bytes:x}, input msg: {msg}");
            // end debug..
            mac.verify_slice(&tag)?;
            Ok(format!("<p><i>{error_message}</i></p>"))
        }
    }
}
