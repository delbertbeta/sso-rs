use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use cookie::Cookie;
use http::{header, HeaderMap, HeaderValue};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    constants::{PARSED_FRONTEND_URL, SESSION_COOKIE_KEY},
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::user::UserModel,
    response::OkResponse,
    util::{decrypt_rsa_content, extract_private_key, verify_password},
};

const SESSION_EXPIRES_TIME: u64 = 60 * 60 * 24 * 15;

#[derive(Deserialize, Validate)]
pub struct LoginParams {
    #[validate(
        required,
        length(min = 1, max = 24),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    username: Option<String>,

    #[validate(required, non_control_character)]
    password: Option<String>,

    #[validate(required, non_control_character)]
    rsa_token: Option<String>,
}

#[derive(Serialize)]
pub struct SuccessResponse {}

pub async fn handler(
    Json(login_params): Json<LoginParams>,
    Extension(store): Extension<RedisSessionStore>,
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<Response, AppError> {
    login_params.validate()?;

    let password = login_params.password.unwrap();
    let username = login_params.username.unwrap();
    let rsa_token = login_params.rsa_token.unwrap();

    let private_key = extract_private_key(&rsa_token, &store).await?;
    let password =
        decrypt_rsa_content(private_key, password)?.ok_or(ServiceError::DecryptPasswordError)?;

    let user = UserModel::new(&conn);
    let user = user
        .find_one_user_by_username_no_related(&username)
        .await?
        .ok_or(ServiceError::LoginFailed)?;

    if !verify_password(&password, &user.salt, &user.password_hash)? {
        return Err(ServiceError::LoginFailed.into());
    }

    let mut session = Session::new();
    session
        .insert("user", UserIdFromSession { user_id: user.id })
        .unwrap();

    session.expire_in(std::time::Duration::from_secs(SESSION_EXPIRES_TIME));
    let token = store.store_session(session).await?.unwrap();

    let cookie = Cookie::build(SESSION_COOKIE_KEY, token)
        .secure(PARSED_FRONTEND_URL.scheme().eq("https"))
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .max_age(cookie::time::Duration::seconds(
            SESSION_EXPIRES_TIME.try_into().unwrap(),
        ))
        .finish();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );
    Ok((StatusCode::OK, headers, OkResponse::new(SuccessResponse {})).into_response())
}
