use crate::routes::utils::attach_flashed_message;
use crate::session_state::TypedSession;
use crate::Request;
use secrecy::{ExposeSecret, Secret};
use tide::StatusCode;
use tide::{Redirect, Response, Result};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(mut req: Request) -> Result {
    let session = TypedSession::from_req(&req);
    if session.get_user_id().is_none() {
        return Ok(Redirect::see_other("/login").into());
    }

    let data: FormData = req.body_form().await.map_err(|mut e| {
        e.set_status(StatusCode::BadRequest);
        e
    })?;
    let hmac_key = &req.state().hmac_secret;
    if data.new_password.expose_secret() != data.new_password_check.expose_secret() {
        let mut response: Response = Redirect::see_other("/admin/password").into();
        let error_msg =
            "You entered two different new passwords - the field values must match.".to_string();
        attach_flashed_message(&mut response, hmac_key, error_msg);
        return Ok(response);
    }

    todo!()
}
