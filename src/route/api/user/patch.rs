use axum::{extract::Extension, Json};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::{
        image::ImageModel,
        user::{UpdateUserInfoParams, UserModel},
    },
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    msg: String,
}

#[derive(Deserialize, Validate)]
pub struct PatchUserParams {
    #[validate(
        required,
        length(min = 1, max = 24),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    nickname: Option<String>,

    #[validate(length(max = 255))]
    self_info: Option<String>,

    #[validate(non_control_character, length(equal = 36))]
    face_id: Option<String>,
}

pub async fn handler(
    Json(patch_user_params): Json<PatchUserParams>,
    user_id_from_session: UserIdFromSession,
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    patch_user_params.validate()?;

    let nickname = patch_user_params.nickname.unwrap();
    let self_info = patch_user_params.self_info.unwrap();
    let face_id = patch_user_params.face_id;

    if let Some(face_id) = &face_id {
        let image = ImageModel::new(&conn);
        let image = image
            .find_one_image_by_id(face_id)
            .await?
            .ok_or(ServiceError::ImageNotFound)?;
        if image.user_id != user_id_from_session.user_id {
            return Err(ServiceError::ImageNotFound.into());
        }
    }

    let user_model = UserModel::new(&conn);

    let user = user_model
        .find_one_user_by_id_no_related(&user_id_from_session.user_id)
        .await?
        .ok_or(ServiceError::NotFound)?;

    user_model
        .update_user_info(
            user.into(),
            UpdateUserInfoParams {
                nickname,
                self_info,
                face_id,
            },
        )
        .await?;

    Ok(OkResponse::new(SuccessResponse {
        msg: "ok".to_string(),
    }))
}
