use crate::routes::utils::get_flashed_message;
use crate::Request;
use tide::http::Cookie;
use tide::{Response, Result};

pub async fn change_password_form(req: Request) -> Result {
    let msg_html = get_flashed_message(&req);
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
    resp.remove_cookie(Cookie::named("_flash"));
    resp.remove_cookie(Cookie::named("tag"));
    Ok(resp)
}
