use http_types::headers;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tide::StatusCode;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, publish_newsletter, subscribe, PublishError};
use crate::State;
use std::net::TcpListener;
use tide::utils::After;
use tide_tracing::TraceMiddleware;

// A warpper for tide::Server to hold the newly built server and its port.
pub struct Application {
    port: u16,
    server: tide::Server<State>,
    listener: TcpListener,
}

impl Application {
    pub fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let server = get_server(
            get_connection_pool(&configuration.database),
            email_client,
            configuration.application.base_url.clone(),
        );
        let listener = TcpListener::bind(format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        ))?;
        let port = listener.local_addr().unwrap().port();

        Ok(Self {
            port,
            server,
            listener,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.listen(self.listener).await
    }
}

fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

fn get_server(db_pool: PgPool, email_client: EmailClient, base_url: String) -> tide::Server<State> {
    let state = State::new(db_pool, email_client, base_url);
    let mut app = tide::with_state(state);
    app.with(After(|mut res: tide::Response| async {
        if let Some(err) = res.downcast_error::<PublishError>() {
            if let PublishError::AuthError(_) = err {
                res.set_status(StatusCode::Unauthorized);
                res.append_header(headers::WWW_AUTHENTICATE, r#"Basic realm="publish""#);
            }
        }
        Ok(res)
    }));
    app.with(TraceMiddleware::new());
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.at("/subscriptions/confirm").get(confirm);
    app.at("/newsletters").post(publish_newsletter);
    app
}
