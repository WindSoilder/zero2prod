use crate::Request;
use tide::Result;

pub async fn health_check(mut _req: Request) -> Result {
    Ok("".into())
}
