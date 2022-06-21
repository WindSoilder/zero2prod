use std::net::TcpListener;

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

// Launch our application in the background ~somehow~
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let _ = async_std::task::spawn(async { zero2prod::get_server().listen(listener).await });
    format!("http://127.0.0.1:{port}")
}
