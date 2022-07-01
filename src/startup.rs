use crate::routes::{health_check, subscribe};

pub fn get_server() -> tide::Server<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app
}
