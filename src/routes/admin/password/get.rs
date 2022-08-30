use crate::routes::utils::verify_cookie;
use crate::session_state::TypedSession;
use crate::Request;
use tide::{Redirect, Response, Result};

pub async fn change_password_form(_req: Request) -> Result {
    let session = TypedSession::from_req(&_req);
    if session.get_user_id().is_none() {
        return Ok(Redirect::see_other("/login").into());
    }
    let msg_html = if verify_cookie(&_req) {
        match _req.cookie("_flash") {
            Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
            None => "".into(),
        }
    } else {
        "".into()
    };
    let body = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">

    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Change Password</title>
    </head>

    <body>
        {msg_html}
        <form action="/admin/password" method="post">
            <label>Current password
                <input type="password" placeholder="Enter current password" name="current_password">
            </label>
            <br>
            <label>New password
                <input type="password" placeholder="Enter new password" name="new_password"> </label>
            <br>
            <label>Confirm new password
                <input type="password" placeholder="Type the new password again" name="new_password_check">
            </label>
            <br>
            <button type="submit">Change password</button>
        </form>
        <p><a href="/admin/dashboard">&lt;- Back</a></p>
    </body>

    </html>"#
    );
    let mut resp: Response = body.into();
    resp.set_content_type("text/html; charset=utf-8");
    Ok(resp)
}
