use crate::Request;
// use crate::{routes::utils::attach_flashed_message, session_state::TypedSession};
use crate::session_state::TypedSession;
use tide::{Redirect, Response, Result};

pub async fn log_out(req: Request) -> Result {
    let session = TypedSession::from_req(&req);
    if session.get_user_id().is_none() {
        Ok(Redirect::see_other("/login").into())
    } else {
        session.log_out();
        let resp: Response = Redirect::see_other("/login").into();
        // let hmac_key = &req.state().hmac_secret;
        // FIXME: I don't know why the attach flashed message doesn't work...keep it for now
        // TODO: investigate it in the future.
        /*
        attach_flashed_message(
            &mut resp,
            hmac_key,
            "You have successfully logged out.".into(),
        );
        */
        Ok(resp)
    }
}
