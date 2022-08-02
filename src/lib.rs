pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;
pub mod telemetry;

use email_client::EmailClient;
use sqlx::PgPool;
pub use startup::get_server;

#[derive(Clone)]
pub struct State {
    connection: PgPool,
    email_client: EmailClient,
}

impl State {
    pub fn new(pg_pool: PgPool, email_client: EmailClient) -> Self {
        State {
            connection: pg_pool,
            email_client,
        }
    }
}

type Request = tide::Request<State>;
