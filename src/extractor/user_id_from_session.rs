use async_redis_session::RedisSessionStore;
use async_session::{async_trait, SessionStore};
use axum::{
    extract::{Extension, FromRequest, RequestParts},
    headers::Cookie,
    TypedHeader,
};
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
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .expect("`RedisSessionStore` extension missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
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

        let user = session
            .get::<UserIdFromSession>("user")
            .ok_or(ServiceError::LoginRequired)?;

        Ok(user)
    }
}
