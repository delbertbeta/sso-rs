use async_redis_session::RedisSessionStore;
use axum::{extract::Extension, Json};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{AppError, ServiceError},
    model::user::{CreateUserParams, UserModel},
    response::OkResponse,
    util::{decrypt_rsa_content, extract_private_key, hash_password},
};

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

    // Except a SHA256 hashed string
    #[validate(required, non_control_character)]
    password: Option<String>,

    #[validate(required, non_control_character)]
    rsa_token: Option<String>,
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

    let private_key = extract_private_key(&rsa_token, &store).await?;
    let password =
        decrypt_rsa_content(private_key, password)?.ok_or(ServiceError::DecryptPasswordError)?;

    if password.len() != 64 {
        return Err(ServiceError::InvalidPasswordLength.into());
    }

    let user_model = UserModel::new(&conn);
    let user = user_model.find_one_user_by_username(&username).await?;

    if user.is_some() {
        return Err(ServiceError::DuplicatedUsername.into());
    }

    let (salt, password_hash) = hash_password(&password, None)?;

    user_model
        .insert_user(CreateUserParams {
            username,
            salt,
            email,
            nickname,
            password_hash,
        })
        .await?;

    Ok(OkResponse::new(SuccessResponse {
        msg: "ok".to_string(),
    }))
}
