use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::get_server;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let server = get_server(PgPool::connect_lazy_with(configuration.database.with_db()));

    server
        .listen(format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        ))
        .await?;
    Ok(())
}
