use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{
    extract::Extension,
    headers::Cookie,
    response::{IntoResponse, Response},
    TypedHeader,
};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use serde::Serialize;

use crate::{
    constants::{PARSED_FRONTEND_URL, SESSION_COOKIE_KEY},
    error::{AppError, ServiceError},
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {}

pub async fn handler(
    Extension(store): Extension<RedisSessionStore>,
    cookie: TypedHeader<Cookie>,
) -> Result<Response, AppError> {
    let session_cookie = &cookie
        .get(SESSION_COOKIE_KEY)
        .ok_or(ServiceError::LoginRequired)?;

    let mut session = store
        .load_session(session_cookie.to_string())
        .await?
        .ok_or(ServiceError::LoginRequired)?;

    session.destroy();

    let cookie = cookie::Cookie::build(SESSION_COOKIE_KEY, "logout")
        .secure(PARSED_FRONTEND_URL.scheme().eq("https"))
        .path("/")
        .http_only(true)
        .same_site(cookie::SameSite::Strict)
        .max_age(cookie::time::Duration::seconds(0))
        .finish();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok((StatusCode::OK, headers, OkResponse::new(SuccessResponse {})).into_response())
}
