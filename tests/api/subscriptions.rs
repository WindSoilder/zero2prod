use crate::helpers::spawn_app;
use serde::{Deserialize, Serialize};
use surf::http::{Method, Url};

#[derive(Serialize, Clone)]
struct Subscription {
    name: String,
    email: String,
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
