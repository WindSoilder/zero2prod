use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::session_state::TypedSession;
use crate::Request;
use http_types::headers;
use secrecy::Secret;
use serde::Deserialize;
use tide::{Redirect, Response, Result, StatusCode};

use super::utils::attach_cookie;

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
                attach_cookie(&mut response, &req.state().hmac_secret, error_msg);
                response.append_header(headers::LOCATION, "/login");
                return Ok(response);
            }
            AuthError::UnexpectedError(_) => {
                return Err(LoginError::UnexpectedError(e.into()).into())
            }
        },
    };
    let mut session = TypedSession::from_req(&req);
    if let Err(e) = session.insert_user_id(user_id) {
        let error = LoginError::UnexpectedError(e.into());
        let error_msg = error.to_string();
        let mut response = Response::new(StatusCode::SeeOther);
        attach_cookie(&mut response, &req.state().hmac_secret, error_msg);
        response.append_header(headers::LOCATION, "/login");
        return Ok(response);
    }
    session.regenerate();

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    Ok(Redirect::see_other("/admin/dashboard").into())
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}
