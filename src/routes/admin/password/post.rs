use crate::session_state::TypedSession;
use crate::Request;
use secrecy::Secret;
use tide::{Redirect, Result};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(req: Request) -> Result {
    let session = TypedSession::from_req(&req);
    if session.get_user_id().is_none() {
        return Ok(Redirect::see_other("/login").into());
    }
    todo!()
}
