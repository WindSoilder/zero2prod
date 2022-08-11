use serde::Deserialize;
use crate::Request;
use tide::Result;

#[derive(Deserialize)]
struct Parameters {
    subscription_token: String
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(req))]
pub async fn confirm(mut req: Request) -> Result {
    let token: Parameters = req.query()?;
    return Ok("".into());
}
