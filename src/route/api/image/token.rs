use anyhow::anyhow;
use chrono::{Duration, Utc};
use qcloud::sts::{get_credential, get_policy, StsResponse};
use serde::Serialize;

use crate::constants::SECRETS;
use crate::{
    constants::ENVS, error::AppError, extractor::user_id_from_session::UserIdFromSession,
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    tmp_secret_id: String,
    tmp_secret_key: String,
    session_token: String,
    start_time: i64,
    expired_time: i64,
}

pub async fn handler(
    _user_id_from_session: UserIdFromSession,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let policy = get_policy(vec![(
        "name/cos:PutObject",
        ENVS.cos_bucket_name.as_str(),
        ENVS.cos_bucket_region.as_str(),
        "images/*",
    )
        .into()]);

    let credential = get_credential(&SECRETS, &policy, &ENVS.cos_bucket_region, 600).await?;

    let credential = match credential.response {
        StsResponse::Success(res) => res,
        StsResponse::Error(e) => return Err(anyhow!("{:?}", e).into()),
    };

    let expire = Utc::now() + Duration::seconds(600);

    Ok(OkResponse::new(SuccessResponse {
        tmp_secret_id: credential.credentials.tmp_secret_id,
        tmp_secret_key: credential.credentials.tmp_secret_key,
        session_token: credential.credentials.token,
        start_time: Utc::now().timestamp(),
        expired_time: expire.timestamp(),
    }))
}
