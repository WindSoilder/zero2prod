use crate::Request;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};
use tide::Response;

pub fn attach_cookie(response: &mut Response, hmac_key: &Secret<String>, error_msg: String) {
    response.insert_cookie(http_types::Cookie::new("_flash", error_msg.clone()));
    // attach hmac_tag to result.
    let msg = format!("_flash={error_msg}");
    let hmac_tag = gen_hmac_tag(hmac_key, msg);
    response.insert_cookie(http_types::Cookie::new("tag", hmac_tag));
}

pub fn verify_cookie(req: &Request) -> bool {
    let error_msg = match req.cookie("_flash") {
        None => return false,
        Some(cookie) => cookie.value().to_string(),
    };

    let hmac_key = &req.state().hmac_secret;
    match req.cookie("tag") {
        None => return false,
        Some(tag) => {
            let tag = match hex::decode(tag.value()) {
                Ok(t) => t,
                Err(_) => return false,
            };
            let msg = format!("_flash={error_msg}");
            verify_hmac_tag(hmac_key, msg, &tag)
        }
    }
}

fn gen_hmac_tag(hmac_key: &Secret<String>, msg: String) -> String {
    let mut mac =
        Hmac::<sha2::Sha256>::new_from_slice(hmac_key.expose_secret().as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    let mac_bytes = mac.finalize().into_bytes();
    format!("{mac_bytes:x}")
}

fn verify_hmac_tag(hmac_key: &Secret<String>, msg: String, input_tag: &[u8]) -> bool {
    let mut mac =
        Hmac::<sha2::Sha256>::new_from_slice(hmac_key.expose_secret().as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    mac.verify_slice(input_tag).map(|_| true).unwrap_or(false)
}
