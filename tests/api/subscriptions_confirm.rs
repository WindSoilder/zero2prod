use crate::helpers::{spawn_app, Subscription};
use surf::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[async_std::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = surf::get(format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), 400);
}

#[async_std::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = Subscription {
        name: Some("le guin".to_string()),
        email: Some("ursula_le_guin@gmail.com".to_string()),
    };

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(&body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Extract the link from one of the rquest fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };
    let raw_confirmation_link = &get_link(&body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    confirmation_link.set_port(Some(app.port)).unwrap();

    // Let's make sure we don't call random APIs on the web.
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // Act
    let response = surf::get(confirmation_link).await.unwrap();

    // Assert
    assert_eq!(response.status(), 200);
}
