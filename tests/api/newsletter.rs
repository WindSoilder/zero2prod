use crate::helpers::{
    assert_is_redirect_to, spawn_app, ConfirmationLinks, Subscription, TestApp,
};
use surf::StatusCode;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[async_std::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange.
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Login
    let login_body =
        serde_json::json!({"username": app.test_user.username, "password": app.test_user.password});
    let _ = app.post_login(&login_body).await;

    // Act - Part 2 - post newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletters(newsletter_request_body).await;
    // Assert
    assert_eq!(response.status(), StatusCode::Ok);

    // Act - Part 3 - clear, logout
    let _ = app.post_logout().await;
}

#[async_std::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    // Act - Part 1 - Login
    let login_body =
        serde_json::json!({"username": app.test_user.username, "password": app.test_user.password});
    let _ = app.post_login(&login_body).await;

    // Act - Part 2 - post newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletters(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status(), StatusCode::Ok);
    // Act - Part 3 - clear, logout
    let _ = app.post_logout().await;
}

#[async_std::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({"content": {"text": "Newsletter body as plain text", "html": "<p>Newsletter body as HTML</p>"}}),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];
    // Act - Part 1 - Login
    let login_body =
        serde_json::json!({"username": app.test_user.username, "password": app.test_user.password});
    let _ = app.post_login(&login_body).await;

    // Act - Part 2 - post newsletter
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        // Assert
        assert_eq!(
            StatusCode::BadRequest,
            response.status(),
            "The API did not fail with 400 Bad Request when thee payload was {}.",
            error_message
        )
    }
    // Act - Part 3 - clear, logout
    let _ = app.post_logout().await;
}

#[async_std::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;

    let response = surf::post(format!("{}/admin/newsletters", &app.address))
        .body_json(&serde_json::json! ({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .unwrap()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_is_redirect_to(&response, "/login");
}

/// Use the public API of the application under test to create an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = Subscription {
        name: Some("le guin".to_string()),
        email: Some("ursula_le_guin@gmail.com".to_string()),
    };
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    let resp = app.post_subscriptions(&body).await;
    if resp.status().is_client_error() || resp.status().is_server_error() {
        panic!("post subscripitons during create_unconfirmed_subscriber shouldn't failed");
    }

    let email_request = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    let resp = surf::get(confirmation_link.html).await.unwrap();
    if resp.status().is_client_error() || resp.status().is_server_error() {
        panic!("post subscripitons during create_unconfirmed_subscriber shouldn't failed");
    }
}
