use axum::extract::{Extension, Path};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::{application::ApplicationModel, application_secret::ApplicationSecretModel},
    response::OkResponse,
    util::mask_secret,
};

#[derive(Serialize)]
struct ResponseApplicationSecret {
    secret: String,
    created_at: String,
    id: i32,
}

#[derive(Deserialize)]
pub struct GetSecretsListQueryParams {
    pub application_id: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    application_secrets: Vec<ResponseApplicationSecret>,
}

pub async fn handler(
    Extension(conn): Extension<DatabaseConnection>,
    Path(url_params): Path<GetSecretsListQueryParams>,
    user_id_from_session: UserIdFromSession,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let application_model = ApplicationModel::new(&conn);
    let application_secret_model = ApplicationSecretModel::new(&conn);

    let application = application_model
        .find_one_application_by_id(url_params.application_id.as_str())
        .await?
        .ok_or(ServiceError::NotFound)?;

    if application.0.creator_id != user_id_from_session.user_id {
        return Err(ServiceError::PermissionDenied.into());
    }

    let secrets = application_secret_model
        .get_secrets_by_application_id(&url_params.application_id)
        .await?;

    let res = SuccessResponse {
        application_secrets: secrets
            .into_iter()
            .map(|secret| ResponseApplicationSecret {
                created_at: secret.created_at.to_string(),
                secret: mask_secret(&secret.secret),
                id: secret.id,
            })
            .collect(),
    };

    Ok(OkResponse::new(res))
}
