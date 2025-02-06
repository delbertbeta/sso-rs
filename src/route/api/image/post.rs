use std::time::Duration as TimeDuration;

use anyhow::anyhow;
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::{
    constants::{ENVS, SUPPORT_IMAGE_TYPE},
    error::AppError,
    extractor::user_id_from_session::UserIdFromSession,
    model::image::{CreateImageParams, ImageModel},
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    image_path: String,
    image_id: String,
    token: TokenResponse,
}

#[derive(Serialize)]
pub struct TokenResponse {
    presigned_url: String,
    start_time: i64,
    expired_time: i64,
}

#[derive(Deserialize, Validate)]
pub struct PostImageParams {
    #[validate(custom(function = "validate_image_type"))]
    image_type: String,
}

fn validate_image_type(image_type: &str) -> Result<(), ValidationError> {
    if SUPPORT_IMAGE_TYPE.contains(image_type) {
        return Ok(());
    }
    return Err(ValidationError::new("ImageType not support"));
}

pub async fn handler<'a>(
    user_id_from_session: UserIdFromSession,
    Extension(conn): Extension<DatabaseConnection>,
    Extension(s3_client): Extension<Client>,
    Json(post_image_params): Json<PostImageParams>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    post_image_params.validate()?;

    let image = ImageModel::new(&conn);

    let id = uuid::Uuid::new_v4().to_string();
    let path = format!("images/{}.{}", id, post_image_params.image_type);

    let presigned_res = s3_client
        .put_object()
        .bucket(&ENVS.bucket_name)
        .key(&path)
        .presigned(
            PresigningConfig::builder()
                .expires_in(TimeDuration::from_secs(60 * 10))
                .build()
                .unwrap(),
        )
        .await;

    match presigned_res {
        Ok(credential) => {
            let expire = Utc::now() + Duration::seconds(600);

            image
                .insert_image(CreateImageParams {
                    user_id: user_id_from_session.user_id,
                    id: &id,
                    path: &path,
                })
                .await?;

            return Ok(OkResponse::new(SuccessResponse {
                image_id: id,
                image_path: path,
                token: TokenResponse {
                    presigned_url: credential.uri().into(),
                    start_time: Utc::now().timestamp(),
                    expired_time: expire.timestamp(),
                },
            }));
        }
        Err(err) => match err.into_service_error() {
            err @ _ => return Err(AppError::UnexpectedError(anyhow!("{:?}", err))),
        },
    }
}
