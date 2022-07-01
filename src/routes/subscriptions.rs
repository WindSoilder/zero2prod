use serde::{Deserialize, Serialize};
use tide::{Request, Result};

#[derive(Clone, Deserialize, Serialize)]
struct SubscribeBody {
    email: String,
    name: String,
}

pub async fn subscribe(mut _req: Request<()>) -> Result {
    let _: SubscribeBody = _req.body_form().await.map_err(|mut e| {
        e.set_status(400);
        e
    })?;
    return Ok("".into());
}
