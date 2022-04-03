use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use openssl::{
    error::ErrorStack,
    rsa::{Padding, Rsa},
};
use pbkdf2::{
    password_hash::{Error, PasswordHasher, SaltString},
    Pbkdf2,
};
use rand_core::OsRng;
use validator::ValidationError;

use crate::{
    constants::RSA_PRIVATE_KEY_REDIS_KEY,
    error::{AppError, ServiceError},
};

pub fn validate_padding_string(val: &str) -> Result<(), ValidationError> {
    let str = String::from(val);
    match str.starts_with(" ") || str.ends_with(" ") {
        true => Err(ValidationError::new("Can't starts or ends with space")),
        false => Ok(()),
    }
}

pub fn decrypt_rsa_content(
    private_key: String,
    content: String,
) -> Result<Option<String>, ErrorStack> {
    let private_key = Rsa::private_key_from_pem(private_key.as_bytes())?;
    let mut buf: Vec<u8> = vec![0; private_key.size() as usize];

    let content = base64::decode(content).unwrap_or(b"".to_vec());

    private_key.private_decrypt(&content, &mut buf, Padding::PKCS1)?;

    let decrypted = String::from_utf8(buf);

    match decrypted {
        Err(_) => Ok(None),
        Ok(val) => Ok(Some(val.trim_matches(char::from(0)).to_string())),
    }
}

pub fn hash_password(password: &String, salt: Option<&String>) -> Result<(String, String), Error> {
    let salt = match salt {
        Some(salt) => SaltString::new(salt)?,
        None => SaltString::generate(&mut OsRng),
    };
    let password_vec = password.as_bytes();
    let password_hash = Pbkdf2
        .hash_password(password_vec, &salt)?
        .hash
        .expect("Get hash value failed");

    let mut password_hash_buffer: Vec<u8> = vec![0; password_hash.b64_len() * 8];

    let password_hash = password_hash.b64_encode(&mut password_hash_buffer)?;

    Ok((salt.to_string(), password_hash.to_string()))
}

pub fn verify_password(
    password: &String,
    salt: &String,
    hashed_pass: &String,
) -> Result<bool, Error> {
    let (_, password_hash) = hash_password(&password, Some(salt))?;
    Ok(password_hash.eq(hashed_pass))
}

pub async fn extract_private_key(
    rsa_token: &String,
    store: &RedisSessionStore,
) -> Result<String, AppError> {
    let session = store.load_session(rsa_token.to_string()).await?;

    let private_key = match session {
        Some(s) => Ok(s.get::<String>(RSA_PRIVATE_KEY_REDIS_KEY).unwrap()),
        None => Err(ServiceError::InvalidRsaToken),
    }?;

    Ok(private_key)
}
