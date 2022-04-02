use async_redis_session::RedisSessionStore;
use async_session::{log::trace, SessionStore};
use axum::{extract::Extension, Json};
use chrono::prelude::*;
use entity::user;
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Pbkdf2,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, NotSet, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use user::Entity as User;
use validator::Validate;

use crate::{
    constants::RSA_PRIVATE_KEY_REDIS_KEY, error::RegisterError, util::decrypt_rsa_content,
};
use crate::{error::AppError, response::OkResponse};

#[derive(Serialize)]
pub struct SuccessResponse {
    msg: String,
}

#[derive(Deserialize, Validate)]
pub struct RegisterParams {
    #[validate(
        required,
        length(min = 1, max = 24),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    username: Option<String>,
    #[validate(
        required,
        length(min = 1, max = 24),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    nickname: Option<String>,
    #[validate(required, email)]
    email: Option<String>,
    #[validate(required, non_control_character)]
    rsa_token: Option<String>,

    // Except a SHA256 hashed string
    #[validate(required, non_control_character)]
    password: Option<String>,
}

pub async fn handler(
    Json(register_params): Json<RegisterParams>,
    Extension(store): Extension<RedisSessionStore>,
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    register_params.validate()?;

    let username = register_params.username.unwrap();
    let password = register_params.password.unwrap();
    let email = register_params.email.unwrap();
    let nickname = register_params.nickname.unwrap();
    let rsa_token = register_params.rsa_token.unwrap();

    let session = store.load_session(rsa_token).await?;

    let private_key = match session {
        Some(s) => Ok(s.get::<String>(RSA_PRIVATE_KEY_REDIS_KEY).unwrap()),
        None => Err(RegisterError::InvalidRsaToken),
    }?;

    let password = decrypt_rsa_content(private_key, password)?;

    if password.is_none() {
        return Err(RegisterError::DecryptPasswordError.into());
    }

    let password = password.unwrap();

    if password.len() != 64 {
        return Err(RegisterError::InvalidPasswordLength.into());
    }

    let user = User::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(&conn)
        .await?;

    if user.is_some() {
        return Err(RegisterError::DuplicatedUsername.into());
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_vec: Vec<u8> = password.into();
    let password_hash = Pbkdf2
        .hash_password(&password_vec, &salt)?
        .hash
        .expect("Get hash value failed");

    let mut password_hash_buffer: Vec<u8> = vec![0; password_hash.b64_len() * 8];

    let password_hash = password_hash.b64_encode(&mut password_hash_buffer)?;

    let new_user = user::ActiveModel {
        id: NotSet,
        username: Set(username),
        salt: Set(salt.to_string()),
        email: Set(Some(email)),
        face_url: NotSet,
        password_hash: Set(password_hash.to_string()),
        nickname: Set(nickname),
        self_info: Set(Some("".to_string())),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
    };

    new_user.insert(&conn).await?;

    Ok(OkResponse::new(SuccessResponse {
        msg: "ok".to_string(),
    }))
}
