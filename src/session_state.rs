use crate::Request;
use tide::sessions::Session;
use uuid::Uuid;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn from_req(req: &Request) -> Self {
        Self(req.session().clone())
    }
    pub fn regenerate(&mut self) {
        self.0.regenerate()
    }

    pub fn insert_user_id(&mut self, user_id: Uuid) -> std::result::Result<(), serde_json::Error> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Option<Uuid> {
        self.0.get(Self::USER_ID_KEY)
    }

    pub fn log_out(mut self) {
        self.0.destroy()
    }
}
