pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod authentication;

use email_client::EmailClient;
use sqlx::PgPool;
pub use startup::Application;

#[derive(Clone)]
pub struct State {
    connection: PgPool,
    email_client: EmailClient,
    base_url: String,
}

impl State {
    pub fn new(pg_pool: PgPool, email_client: EmailClient, base_url: String) -> Self {
        State {
            connection: pg_pool,
            email_client,
            base_url,
        }
    }
}

type Request = tide::Request<State>;
