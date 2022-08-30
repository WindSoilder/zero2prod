use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::Request;
use hmac::{Hmac, Mac};
use http_types::headers;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use tide::{Redirect, Response, Result, StatusCode};

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

pub async fn login(mut req: Request) -> Result {
    let form_data: FormData = req.body_form().await.map_err(|mut e| {
        e.set_status(StatusCode::BadRequest);
        e
    })?;
    let pool = &req.state().connection;
    let credentials = Credentials {
        username: form_data.username,
        password: form_data.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let validate_result = validate_credentials(credentials, pool).await;
    let user_id = match validate_result {
        Ok(user_id) => user_id,
        Err(e) => match e {
            AuthError::InvalidCredentials(_) => {
                let err = LoginError::AuthError(e.into());
                let error_msg = err.to_string();
                let mut response = Response::new(StatusCode::SeeOther);
                response.insert_cookie(http_types::Cookie::new("_flash", error_msg.clone()));
                // attach hmac_tag to result.
                let msg = format!("_flash={error_msg}");
                let secret = &req.state().hmac_secret.expose_secret().as_bytes();
                let hmac_tag = {
                    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
                    mac.update(msg.as_bytes());
                    let mac_bytes = mac.finalize().into_bytes();
                    format!("{mac_bytes:x}")
                };
                response.insert_cookie(http_types::Cookie::new("tag", hmac_tag));
                response.append_header(headers::LOCATION, "/login");
                return Ok(response);
            }
            AuthError::UnexpectedError(_) => {
                return Err(LoginError::UnexpectedError(e.into()).into())
            }
        },
    };
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    Ok(Redirect::see_other("/").into())
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}
