use crate::Request;
use tide::{Response, Result};

pub async fn login_form(_req: Request) -> Result {
    let mut resp: Response = include_str!("login.html").into();
    resp.set_content_type("text/html; charset=utf-8");
    Ok(resp)
}
