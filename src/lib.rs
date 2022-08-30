pub mod authentication;
pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod session_state;

use email_client::EmailClient;
use secrecy::Secret;
use sqlx::PgPool;
pub use startup::Application;

#[derive(Clone)]
pub struct State {
    connection: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
}

impl State {
    pub fn new(
        pg_pool: PgPool,
        email_client: EmailClient,
        base_url: String,
        hmac_secret: Secret<String>,
    ) -> Self {
        State {
            connection: pg_pool,
            email_client,
            base_url,
            hmac_secret,
        }
    }
}

type Request = tide::Request<State>;
