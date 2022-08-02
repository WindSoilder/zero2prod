use sqlx::PgPool;

use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use crate::State;
use tide_tracing::TraceMiddleware;

pub fn get_server(db_pool: PgPool, email_client: EmailClient) -> tide::Server<State> {
    let state = State::new(db_pool, email_client);
    let mut app = tide::with_state(state);
    app.with(TraceMiddleware::new());
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app
}
