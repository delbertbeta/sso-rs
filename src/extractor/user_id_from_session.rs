use async_redis_session::RedisSessionStore;
use async_session::{async_trait, SessionStore};
use axum::{
    extract::{Extension, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{headers::Cookie, TypedHeader};
use serde::{Deserialize, Serialize};

use crate::{
    constants::SESSION_COOKIE_KEY,
    error::{AppError, ServiceError},
};

#[derive(Serialize, Deserialize)]
pub struct UserIdFromSession {
    pub user_id: i32,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserIdFromSession
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request_parts(parts, state)
            .await
            .expect("`RedisSessionStore` extension missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request_parts(parts, state)
            .await
            .unwrap();

        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(SESSION_COOKIE_KEY))
            .ok_or(ServiceError::LoginRequired)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await?
            .ok_or(ServiceError::LoginRequired)?;

        if session.is_destroyed() {
            return Err(ServiceError::LoginRequired.into());
        }

        let user = session
            .get::<UserIdFromSession>("user")
            .ok_or(ServiceError::LoginRequired)?;

        Ok(user)
    }
}
