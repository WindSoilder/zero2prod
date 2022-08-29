use crate::Request;
use tide::{Response, Result};

pub async fn home(mut _req: Request) -> Result {
    let mut resp: Response = include_str!("home.html").into();
    resp.set_content_type("text/html; charset=utf-8");
    Ok(resp)
}
