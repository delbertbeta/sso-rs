use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use pilota::AHashMap;
use sea_orm::DatabaseConnection;
use volo_gen::sso::rs::User;

use crate::{
    constants::ENVS,
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::user::UserModel,
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
        req: ::volo_grpc::Request<volo_gen::sso::rs::GetUserRequest>,
    ) -> ::std::result::Result<
        ::volo_grpc::Response<volo_gen::sso::rs::GetUserResponse>,
        ::volo_grpc::Status,
    > {
        let user = UserModel::new(&self.conn);
        let mut users = AHashMap::<i32, User>::new();

        for user_id in &req.get_ref().user_id {
            match user.find_one_user_by_id(user_id).await {
                Ok(query_res) => {
                    if query_res.is_some() {
                        let (user_model, face_model) = query_res.unwrap();
                        let face_url: Option<String> = face_model
                            .and_then(|f| Some(format!("{}{}", ENVS.cdn_base_url, f.path)));

                        users.insert(
                            user_id.clone(),
                            User {
                                id: user_model.id,
                                username: user_model.username.into(),
                                face_url: face_url.unwrap_or("".into()).into(),
                                self_info: user_model.self_info.unwrap_or("".into()).into(),
                                nickname: user_model.nickname.into(),
                            },
                        );
                    }
                }
                Err(_) => (),
            }
        }

        Ok(::volo_grpc::Response::new(
            volo_gen::sso::rs::GetUserResponse { users },
        ))
    }
}
