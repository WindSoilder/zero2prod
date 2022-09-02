use crate::session_state::TypedSession;
use tide::{Middleware, Next, Redirect, Result};
#[derive(Default)]
pub struct RequiredLoginMiddleware;

pub struct UserId(pub uuid::Uuid);

#[tide::utils::async_trait]
impl<S: Clone + Send + Sync + 'static> Middleware<S> for RequiredLoginMiddleware {
    async fn handle(&self, mut req: tide::Request<S>, next: Next<'_, S>) -> Result {
        let req_path = req.url().path();
        if [
            "/admin/dashboard",
            "/admin/password",
            "/admin/logout",
            "/admin/newsletters",
        ]
        .contains(&req_path)
        {
            let session = TypedSession::from_req(&req);
            let user_id = match session.get_user_id() {
                None => return Ok(Redirect::see_other("/login").into()),
                Some(user_id) => user_id,
            };
            req.set_ext(UserId(user_id));
        }
        let res = next.run(req).await;
        Ok(res)
    }
}
