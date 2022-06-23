use serde::{Deserialize, Serialize};
use tide::Request;

pub fn get_server() -> tide::Server<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app
}

async fn health_check(mut _req: Request<()>) -> tide::Result {
    println!("receive request;");
    Ok("".into())
}

#[derive(Clone, Deserialize, Serialize)]
struct SubscribeBody {
    email: String,
    name: String,
}

async fn subscribe(mut _req: Request<()>) -> tide::Result {
    let _: SubscribeBody = _req.body_form().await.map_err(|mut e| {
        e.set_status(400);
        e
    })?;
    return Ok("".into());
}
