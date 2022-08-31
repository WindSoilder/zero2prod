use crate::Request;
use tide::{Response, Result};

pub async fn newsletter_form(_req: Request) -> Result {
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <form action="/admin/newsletters" method="post>
        <label>Title <input type="text" placeholder="Enter Title" name="title"> </label>
        <label>Content <input type="text" name="content"></label>
    </form>
</body>
</html>"#
    );
    let mut resp: Response = body.into();
    resp.set_content_type("text/html; charset=utf-8");
    Ok(resp)
}
