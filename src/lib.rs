use tide::Request;

pub fn get_server() -> tide::Server<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app
}

async fn health_check(mut _req: Request<()>) -> tide::Result {
    Ok("".into())
}
