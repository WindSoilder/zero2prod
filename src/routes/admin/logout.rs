use crate::routes::utils::attach_flashed_message;
use crate::session_state::TypedSession;
use crate::Request;
use tide::{Redirect, Response, Result};

pub async fn log_out(req: Request) -> Result {
    let session = TypedSession::from_req(&req);
    if session.get_user_id().is_none() {
        Ok(Redirect::see_other("/login").into())
    } else {
        session.log_out();
        let mut resp: Response = Redirect::see_other("/login").into();
        let hmac_key = &req.state().hmac_secret;
        attach_flashed_message(
            &mut resp,
            hmac_key,
            "You have successfully logged out.".into(),
        );

        Ok(resp)
    }
}
