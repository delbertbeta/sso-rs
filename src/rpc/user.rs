use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use sea_orm::DatabaseConnection;

use crate::{
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
};

pub struct S {
    conn: DatabaseConnection,
    session_store: RedisSessionStore,
}

impl S {
    pub fn new(conn: DatabaseConnection, session_store: RedisSessionStore) -> Self {
        Self {
            conn,
            session_store,
        }
    }
}

impl volo_gen::sso::rs::UserService for S {
    async fn get_user_id_by_cookie(
        &self,
        req: ::volo_grpc::Request<volo_gen::sso::rs::GetUserIdByCookieRequest>,
    ) -> ::std::result::Result<
        ::volo_grpc::Response<volo_gen::sso::rs::GetUserIdByCookieResponse>,
        ::volo_grpc::Status,
    > {
        let session = self
            .session_store
            .load_session(req.get_ref().cookie.to_string())
            .await?
            .ok_or(AppError::from(ServiceError::LoginRequired))?;

        if session.is_destroyed() {
            return Err(AppError::from(ServiceError::LoginRequired).into());
        }

        let user = session
            .get::<UserIdFromSession>("user")
            .ok_or(AppError::from(ServiceError::LoginRequired))?;

        ::std::result::Result::Ok(::volo_grpc::Response::new(
            volo_gen::sso::rs::GetUserIdByCookieResponse {
                user_id: user.user_id,
            },
        ))
    }

    async fn get_user(
        &self,
        _req: ::volo_grpc::Request<volo_gen::sso::rs::GetUserRequest>,
    ) -> ::std::result::Result<
        ::volo_grpc::Response<volo_gen::sso::rs::GetUserResponse>,
        ::volo_grpc::Status,
    > {
        ::std::result::Result::Ok(::volo_grpc::Response::new(Default::default()))
    }
}
