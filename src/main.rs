use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::get_server;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let server = get_server(
        PgPool::connect(&configuration.database.connection_string().expose_secret())
            .await
            .unwrap(),
    );

    server
        .listen(format!("127.0.0.1:{}", configuration.application_port))
        .await?;
    Ok(())
}
