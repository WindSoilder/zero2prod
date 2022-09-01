use crate::routes::utils::get_flashed_message;
use crate::Request;
use tide::http::Cookie;
use tide::{Response, Result};

pub async fn newsletter_form(req: Request) -> Result {
    let message = get_flashed_message(&req);
    let body = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Publish Newsletter Issue</title>
        </head>
        <body>
            {message}
            <form action="/admin/newsletters" method="post">
                <label>Title:<br>
                    <input
                        type="text"
                        placeholder="Enter the issue title"
                        name="title"
                    >
                </label>
                <br>
                <label>Plain text content:<br>
                    <textarea
                        placeholder="Enter the content in plain text"
                        name="text_content"
                        rows="20"
                        cols="50"
                    ></textarea>
                </label>
                <br>
                <label>HTML content:<br>
                    <textarea
                        placeholder="Enter the content in HTML format"
                        name="html_content"
                        rows="20"
                        cols="50"
                    ></textarea>
                </label>
                <br>
                <button type="submit">Publish</button>
            </form>
            <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
        </html>"#,
    );
    let mut resp: Response = body.into();
    resp.set_content_type("text/html; charset=utf-8");
    resp.remove_cookie(Cookie::named("_flash"));
    resp.remove_cookie(Cookie::named("tag"));
    Ok(resp)
}
