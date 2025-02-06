use axum::{extract::Path, Extension};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{ENVS, SECRETS},
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::image::ImageModel,
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {}

#[derive(Deserialize)]
pub struct PatchImageUrlParams {
    pub image_id: String,
}

#[axum_macros::debug_handler]
pub async fn handler(
    user_id_from_session: UserIdFromSession,
    Path(url_params): Path<PatchImageUrlParams>,
    Extension(conn): Extension<DatabaseConnection>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let image = ImageModel::new(&conn);

    let user_id = user_id_from_session.user_id;
    let image_id = url_params.image_id;

    let image_model = image
        .find_one_image_by_id(&image_id)
        .await?
        .ok_or(ServiceError::NotFound)?;

    if image_model.user_id.ne(&user_id) {
        return Err(ServiceError::PermissionDenied.into());
    }

    if image_model.uploaded.unwrap_or(0) != 0 {
        return Ok(OkResponse::new(SuccessResponse {}));
    }

    let res = qcloud::cos::object::head(
        &SECRETS,
        &ENVS.bucket_name,
        &ENVS.bucket_region,
        &image_model.path,
    )
    .await?;

    if res.status() != 200 {
        return Err(ServiceError::NotFound.into());
    }

    image.set_uploaded(image_model.into(), true).await?;

    Ok(OkResponse::new(SuccessResponse {}))
}
