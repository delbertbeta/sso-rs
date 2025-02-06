use anyhow::anyhow;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use qcloud::sts::{get_credential, get_policy, StsResponse};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::{
    constants::{ENVS, SECRETS, SUPPORT_IMAGE_TYPE},
    error::AppError,
    extractor::user_id_from_session::UserIdFromSession,
    model::image::{CreateImageParams, ImageModel},
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    image_path: String,
    image_id: String,
    bucket: &'static str,
    region: &'static str,
    token: TokenResponse,
}

#[derive(Serialize)]
pub struct TokenResponse {
    tmp_secret_id: String,
    tmp_secret_key: String,
    session_token: String,
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
    Json(post_image_params): Json<PostImageParams>,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    post_image_params.validate()?;

    let image = ImageModel::new(&conn);

    let id = uuid::Uuid::new_v4().to_string();
    let path = format!("images/{}.{}", id, post_image_params.image_type);

    let policy = get_policy(vec![(
        "name/cos:PutObject",
        ENVS.bucket_name.as_str(),
        ENVS.bucket_region.as_str(),
        path.as_str(),
    )
        .into()]);

    let credential = get_credential(&SECRETS, &policy, &ENVS.bucket_region, 600).await?;

    let credential = match credential.response {
        StsResponse::Success(res) => res,
        StsResponse::Error(e) => return Err(anyhow!("{:?}", e).into()),
    };

    let expire = Utc::now() + Duration::seconds(600);

    image
        .insert_image(CreateImageParams {
            user_id: user_id_from_session.user_id,
            id: &id,
            path: &path,
        })
        .await?;

    Ok(OkResponse::new(SuccessResponse {
        image_id: id,
        image_path: path,
        bucket: &ENVS.bucket_name,
        region: &ENVS.bucket_region,
        token: TokenResponse {
            tmp_secret_id: credential.credentials.tmp_secret_id,
            tmp_secret_key: credential.credentials.tmp_secret_key,
            session_token: credential.credentials.token,
            start_time: Utc::now().timestamp(),
            expired_time: expire.timestamp(),
        },
    }))
}
