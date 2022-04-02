use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::Extension;
use openssl::rsa::Rsa;
use serde::Serialize;

use crate::{constants::RSA_PRIVATE_KEY_REDIS_KEY, error::AppError, response::OkResponse};

const RSA_EXPIRES_TIME: u64 = 10 * 60;

#[derive(Serialize)]
pub struct SuccessResponse {
    public_key: String,
    expires: u64,
    token: String,
}

pub async fn handler(
    Extension(store): Extension<RedisSessionStore>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let bits = 1024;
    let rsa = Rsa::generate(bits)?;

    let private_key = rsa.private_key_to_pem()?;
    let public_key = rsa.public_key_to_pem()?;

    let private_key = String::from_utf8(private_key).unwrap();
    let public_key = String::from_utf8(public_key).unwrap();

    let mut session = Session::new();
    session
        .insert(RSA_PRIVATE_KEY_REDIS_KEY, private_key)
        .unwrap();

    session.expire_in(std::time::Duration::from_secs(RSA_EXPIRES_TIME));
    let token = store.store_session(session).await?.unwrap();

    Ok(OkResponse::new(SuccessResponse {
        expires: RSA_EXPIRES_TIME,
        public_key,
        token,
    }))
}
