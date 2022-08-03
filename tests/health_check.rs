use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use surf::http::{Method, Url};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

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

#[derive(Serialize, Clone)]
struct Subscription {
    name: String,
    email: String,
}

#[async_std::test]
async fn health_check_works() {
    // Arrange.
    let test_app = spawn_app().await;
    // We need to bring in `surf`

    let resp = surf::get(format!("{}/health_check", test_app.address))
        .await
        .expect("Failed to execute request");
    assert!(resp.status().is_success());
}

#[async_std::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;

    let url = Url::parse(&format!("{}/subscriptions", test_app.address))
        .expect("failed to parse url address");

    let mut request = surf::Request::builder(Method::Post, url).build();
    request
        .body_form(&Subscription {
            name: "le guin".to_string(),
            email: "ursula_le_guin@gmail.com".to_string(),
        })
        .unwrap();

    let client = surf::client();
    let response = client
        .send(request)
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), 200);
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[async_std::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    #[derive(Clone, Serialize, Deserialize)]
    struct Subscription {
        name: Option<String>,
        email: Option<String>,
    }
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        (
            Subscription {
                name: Some("le guin".to_string()),
                email: None,
            },
            "missing the email",
        ),
        (
            Subscription {
                name: None,
                email: Some("ursula_le_guin@gmail.com".to_string()),
            },
            "missing the name",
        ),
        (
            Subscription {
                name: None,
                email: None,
            },
            "missing both name and email",
        ),
    ];
    let url = Url::parse(&format!("{}/subscriptions", test_app.address))
        .expect("failed to parse url address");
    let client = surf::client();
    for (invalid_body, error_message) in test_cases {
        // Act
        let mut request = surf::Request::builder(Method::Post, url.clone()).build();
        request.body_form(&invalid_body).unwrap();
        let response = client
            .send(request)
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            response.status(),
            400,
            // Additional customized error message on test failure
            "The API did not fail with 400 Bad Request when payload was {}.",
            error_message
        );
    }
}

#[async_std::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    #[derive(Clone, Serialize, Deserialize)]
    struct Subscription {
        name: Option<String>,
        email: Option<String>,
    }
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            Subscription {
                name: Some("".to_string()),
                email: Some("ursula_le_guin@gmail.com".to_string()),
            },
            "empty name",
        ),
        (
            Subscription {
                name: Some("Ursula".to_string()),
                email: Some("".to_string()),
            },
            "empty email",
        ),
        (
            Subscription {
                name: Some("Ursula".to_string()),
                email: Some("not-an-email".to_string()),
            },
            "invalid email",
        ),
    ];
    let url =
        Url::parse(&format!("{}/subscriptions", app.address)).expect("failed to parse url address");
    let client = surf::client();

    for (body, description) in test_cases {
        // Act
        let mut request = surf::Request::builder(Method::Post, url.clone()).build();
        request.body_form(&body).unwrap();
        let response = client
            .send(request)
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            response.status(),
            400,
            "The API did not return a 200 OK when the payload was {}.",
            description
        )
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launch our application in the background ~somehow~
async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let pool_for_server = connection_pool.clone();
    // Build a new email client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );
    let _ = async_std::task::spawn(async {
        zero2prod::get_server(pool_for_server, email_client)
            .listen(listener)
            .await
    });
    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
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
