use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use surf::http::{Method, Url};
use zero2prod::configuration::get_configuration;

#[derive(Serialize, Clone)]
struct Subscription {
    name: String,
    email: String,
}

#[async_std::test]
async fn health_check_works() {
    // Arrange.
    let addr = spawn_app();
    // We need to bring in `surf`
    let resp = surf::get(format!("{addr}/health_check"))
        .await
        .expect("Failed to execute request");
    assert!(resp.status().is_success());
}

#[async_std::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let url =
        Url::parse(&format!("{app_address}/subscriptions")).expect("failed to parse url address");

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
        .fetch_one(&mut connection)
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
    let app_address = spawn_app();
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
    let url =
        Url::parse(&format!("{app_address}/subscriptions")).expect("failed to parse url address");
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

// Launch our application in the background ~somehow~
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let _ = async_std::task::spawn(async { zero2prod::get_server().listen(listener).await });
    format!("http://127.0.0.1:{port}")
}
