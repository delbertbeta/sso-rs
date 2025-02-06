use anyhow::anyhow;
use aws_sdk_s3::{operation::head_object::HeadObjectError, Client};
use axum::{extract::Path, Extension};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    constants::ENVS,
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
    Extension(s3_client): Extension<Client>,
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

    let res = s3_client
        .head_object()
        .bucket(&ENVS.bucket_name)
        .key(&image_model.path)
        .send()
        .await;

    match res {
        Ok(_) => {
            image.set_uploaded(image_model.into(), true).await?;

            return Ok(OkResponse::new(SuccessResponse {}));
        }
        Err(err) => match err.into_service_error() {
            HeadObjectError::NotFound(_) => return Err(ServiceError::NotFound.into()),
            err @ _ => return Err(AppError::UnexpectedError(anyhow!("{:?}", err))),
        },
    }
}
