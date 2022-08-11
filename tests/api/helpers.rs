use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use surf::Url;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: &Subscription) -> surf::Response {
        let url = Url::parse(&format!("{}/subscriptions", self.address))
            .expect("failed to parse url address");

        let client = surf::client();
        let mut request = surf::post(url).build();
        request.body_form(&body).unwrap();
        client
            .send(request)
            .await
            .expect("Failed to execute request.")
    }
}

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// Launch our application in the background ~somehow~
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.application.port = 0;
        c.database.database_name = Uuid::new_v4().to_string();
        c.email_client.base_url = email_server.uri();
        c
    };
    let connection_pool = configure_database(&configuration.database).await;

    let application = zero2prod::Application::build(configuration.clone())
        .expect("initialize application should success");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);
    let _ = async_std::task::spawn(application.run_until_stopped());
    TestApp {
        address,
        db_pool: connection_pool,
        email_server,
        port: application_port,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create databse.
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
