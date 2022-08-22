use crate::helpers::{spawn_app, Subscription, TestApp};
use surf::{StatusCode, Url};
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
    let url =
        Url::parse(&format!("{}/newsletters", app.address)).expect("failed to parse url address");
    let response = surf::post(url)
        .body_json(&newsletter_request_body)
        .expect("Failed to set body json")
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::Ok);
}

/// Use the public API of the application under test to create an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) {
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
}
