use crate::helpers::{assert_is_redirect_to, spawn_app};
use http_types::headers;

#[async_std::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let login_body =
        serde_json::json!({"username": "random-username", "password": "random-password"});
    let response = app.post_login(&login_body).await;

    let flash_cookie = response.header(headers::SET_COOKIE).unwrap();
    assert!(flash_cookie
        .into_iter()
        .find(|x| x.as_str() == "_flash=Authentication%20failed")
        .is_some());

    assert_is_redirect_to(&response, "/login");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    println!("debug: {}", html_page);
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
    // Act - part 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[async_std::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    let app = spawn_app().await;

    // Act - Part 1 - Login
    let login_body =
        serde_json::json!({"username": app.test_user.username, "password": app.test_user.password});
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_admin_dashboard().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}
