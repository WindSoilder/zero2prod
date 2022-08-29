use crate::Request;
use secrecy::Secret;
use serde::Deserialize;
use tide::{Redirect, Result};

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

pub async fn login(_req: Request) -> Result {
    Ok(Redirect::see_other("/").into())
}
