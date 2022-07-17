use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::configuration::get_configuration;
use zero2prod::get_server;

#[async_std::main]
async fn main() -> tide::Result<()> {
    LogTracer::init().expect("failed to initialize log tracer");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subsceiber");

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
