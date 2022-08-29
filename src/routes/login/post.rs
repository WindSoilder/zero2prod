use crate::Request;
use tide::{Redirect, Result};

pub async fn login(_req: Request) -> Result {
    Ok(Redirect::see_other("/").into())
}
