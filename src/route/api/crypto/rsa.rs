use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::Extension;
use rand::rngs::OsRng;
use rsa::{
    pkcs1::{ToRsaPrivateKey, ToRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use serde::Serialize;

use crate::{constants::RSA_PRIVATE_KEY_REDIS_KEY, error::AppError, response::OkResponse};

const RSA_EXPIRES_TIME: u64 = 10 * 60;

#[derive(Serialize)]
pub struct SuccessResponse {
    public_key: String,
    expires: u64,
}

pub async fn handler(
    Extension(store): Extension<RedisSessionStore>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let mut rng = OsRng;
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits)?;
    let public_key = RsaPublicKey::from(&private_key);

    let private_key_str = private_key
        .to_pkcs1_pem()
        .map_err(|e| AppError::UnexpectedError(anyhow::Error::new(e)))?;

    let public_key_str = public_key
        .to_pkcs1_pem()
        .map_err(|e| AppError::UnexpectedError(anyhow::Error::new(e)))?;

    let mut session = Session::new();
    session
        .insert(RSA_PRIVATE_KEY_REDIS_KEY, private_key_str.to_string())
        .unwrap();

    session.expire_in(std::time::Duration::from_secs(RSA_EXPIRES_TIME));
    store.store_session(session).await?;

    Ok(OkResponse::new(SuccessResponse {
        expires: RSA_EXPIRES_TIME,
        public_key: public_key_str.to_string(),
    }))
}
