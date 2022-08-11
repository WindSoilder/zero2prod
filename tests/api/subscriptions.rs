use crate::helpers::{spawn_app, Subscription};
use serde::{Deserialize, Serialize};
use surf::http::{Method, Url};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[async_std::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app
        .post_subscriptions(&Subscription {
            name: Some("le guin".to_string()),
            email: Some("ursula_le_guin@gmail.com".to_string()),
        })
        .await;

    assert_eq!(response.status(), 200);
}

#[async_std::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let test_app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app
        .post_subscriptions(&Subscription {
            name: Some("le guin".to_string()),
            email: Some("ursula_le_guin@gmail.com".to_string()),
        })
        .await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[async_std::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
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
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(&invalid_body).await;
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

#[async_std::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = Subscription {
        name: Some("le guin".to_string()),
        email: Some("ursula_le_guin@gmail.com".to_string()),
    };

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(&body).await;
}

#[async_std::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = Subscription {
        name: Some("le guin".to_string()),
        email: Some("ursula_le_guin@gmail.com".to_string()),
    };

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // We are not setting an expectation here anymore like
        // `subscribe_sends_a_confirmation_email_for_valid_data`
        // The test is focused on another aspect of the app behavior
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(&body).await;

    // Assert
    // Get the first intercepted request.
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // The two links should be identical.
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}
