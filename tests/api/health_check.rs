use crate::helpers::spawn_app;

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
