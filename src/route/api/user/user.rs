use axum::extract::Extension;
use sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::{
    constants::ENVS,
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::user::UserModel,
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub face_url: Option<String>,
    pub nickname: String,
    pub self_info: Option<String>,
}

pub async fn handler(
    user_id_from_session: UserIdFromSession,
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let user = UserModel::new(&conn);
    let (user, face) = user
        .find_one_user_by_id(&user_id_from_session.user_id)
        .await?
        .ok_or(ServiceError::NotFound)?;

    let face_url: Option<String> =
        face.and_then(|f| Some(format!("{}{}", ENVS.cdn_base_url, f.path)));

    Ok(OkResponse::new(SuccessResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        face_url,
        nickname: user.nickname,
        self_info: user.self_info,
    }))
}
