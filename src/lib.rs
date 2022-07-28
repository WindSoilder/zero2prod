pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod domain;

use sqlx::PgPool;
pub use startup::get_server;

#[derive(Clone)]
pub struct State {
    connection: PgPool,
}

impl State {
    pub fn new(pg_pool: PgPool) -> Self {
        State {
            connection: pg_pool,
        }
    }
}

type Request = tide::Request<State>;
