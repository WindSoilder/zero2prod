use crate::helpers::{attach_basic_auth, spawn_app, ConfirmationLinks, Subscription, TestApp};
use surf::StatusCode;
use uuid::Uuid;
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

    // Act
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

    // Act
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
    assert_eq!(StatusCode::Unauthorized, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response
            .header("WWW-Authenticate")
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
    );
}

#[async_std::test]
async fn non_existing_user_is_rejected() {
    // Arrange
    let app = spawn_app().await;
    // Random credentials.
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let mut req = surf::post(&format!("{}/admin/newsletters", app.address)).build();
    attach_basic_auth(&mut req, &username, &password);
    req.body_json(&serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    }))
    .unwrap();
    let resp = surf::client()
        .send(req)
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(StatusCode::Unauthorized, resp.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        resp.header("WWW-Authenticate").unwrap().as_str()
    )
}

#[async_std::test]
async fn invalid_password_is_rejected() {
    // Arrange
    let app = spawn_app().await;
    let username = &app.test_user.username;
    // Random password
    let password = Uuid::new_v4().to_string();
    assert_ne!(app.test_user.password, password);

    let mut req = surf::post(&format!("{}/admin/newsletters", app.address)).build();
    attach_basic_auth(&mut req, &username, &password);
    req.body_json(&serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    }))
    .unwrap();
    let resp = surf::client()
        .send(req)
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(StatusCode::Unauthorized, resp.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        resp.header("WWW-Authenticate").unwrap().as_str()
    )
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
