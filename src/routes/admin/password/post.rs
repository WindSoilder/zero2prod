use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::login_middleware::UserId;
use crate::routes::admin::dashboard::get_username;
use crate::routes::utils::attach_flashed_message;
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
    let user_id = req
        .ext::<UserId>()
        .expect("request session not initialized, did you enable crate::login_middleware::RequiredLoginMiddleware")
        .0;

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

    let pool = &req.state().connection;
    let username = get_username(user_id, &pool).await?;
    let credentials = Credentials {
        username,
        password: data.current_password.clone(),
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                let mut response: Response = Redirect::see_other("/admin/password").into();
                attach_flashed_message(
                    &mut response,
                    hmac_key,
                    "The current password is incorrect".into(),
                );
                Ok(response)
            }
            _ => Err(e.into()),
        };
    }

    crate::authentication::change_password(user_id, data.new_password, &pool).await?;
    let mut resp: Response = Redirect::see_other("/admin/password").into();
    attach_flashed_message(
        &mut resp,
        hmac_key,
        "Your password has been changed.".to_string(),
    );
    Ok(resp)
}
