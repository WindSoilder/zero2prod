use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use http_types::StatusCode;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use surf::Url;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: surf::Client,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash)
            ValueS ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: Url,
    pub plain_text: Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: &Subscription) -> surf::Response {
        let url = Url::parse(&format!("{}/subscriptions", self.address))
            .expect("failed to parse url address");

        let mut request = surf::post(url).build();
        request.body_form(&body).unwrap();
        self.api_client
            .send(request)
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> surf::Response {
        let url = Url::parse(&format!("{}/newsletters", self.address))
            .expect("failed to parse url address");
        let mut request = surf::post(url).build();
        request.body_json(&body).unwrap();
        let (username, password) = (&self.test_user.username, &self.test_user.password);
        attach_basic_auth(&mut request, username, password);
        self.api_client
            .send(request)
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> surf::Response
    where
        Body: serde::Serialize,
    {
        let url =
            Url::parse(&format!("{}/login", &self.address)).expect("failed to parse url address");

        let mut request = surf::post(url).build();
        request.body_form(body).unwrap();
        self.api_client
            .send(request)
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_login_html(&self) -> String {
        let url =
            Url::parse(&format!("{}/login", &self.address)).expect("failed to parse url address");
        let request = surf::get(url).build();
        self.api_client
            .send(request)
            .await
            .expect("Failed to execute requiest")
            .body_string()
            .await
            .unwrap()
    }

    pub async fn get_admin_dashboard(&self) -> String {
        let url = Url::parse(&format!("{}/admin/dashboard", &self.address))
            .expect("failed to parse url address");
        let request = surf::get(url).build();
        self.api_client
            .send(request)
            .await
            .expect("Failed to execute request")
            .body_string()
            .await
            .unwrap()
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        // Parse the body as JSON, starting from raw bytes
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }
}

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// Launch our application in the background ~somehow~
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.application.port = 0;
        c.database.database_name = Uuid::new_v4().to_string();
        c.email_client.base_url = email_server.uri();
        c
    };
    let connection_pool = configure_database(&configuration.database).await;

    let application = zero2prod::Application::build(configuration.clone())
        .expect("initialize application should success");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);
    let _ = async_std::task::spawn(application.run_until_stopped());
    let client = surf::client().with(surf_cookie_middleware::CookieMiddleware::new());
    let test_app = TestApp {
        address,
        db_pool: connection_pool,
        email_server,
        port: application_port,
        test_user: TestUser::generate(),
        api_client: client,
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create databse.
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

pub fn attach_basic_auth(req: &mut surf::Request, name: &str, password: &str) {
    let encode_credentials =
        base64::encode_config(format!("{}:{}", name, password), base64::STANDARD);
    req.append_header(
        http_types::headers::AUTHORIZATION,
        format!("Basic {encode_credentials}"),
    )
}

pub fn assert_is_redirect_to(response: &surf::Response, location: &str) {
    assert_eq!(response.status(), StatusCode::SeeOther);
    assert_eq!(response.header("Location").unwrap().as_str(), location)
}
