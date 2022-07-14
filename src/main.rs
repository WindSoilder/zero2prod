use sqlx::PgPool;
use std::env;
use std::str::FromStr;
use tide::log::LevelFilter;
use zero2prod::configuration::get_configuration;
use zero2prod::get_server;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let level = env::var("RUST_LOG").unwrap_or("info".to_string());
    let log_level = LevelFilter::from_str(&level).unwrap_or(LevelFilter::Info);
    tide::log::with_level(log_level);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let server = get_server(
        PgPool::connect(&configuration.database.connection_string())
            .await
            .unwrap(),
    );

    server
        .listen(format!("127.0.0.1:{}", configuration.application_port))
        .await?;
    Ok(())
}
