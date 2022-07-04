use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::get_server;

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
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
