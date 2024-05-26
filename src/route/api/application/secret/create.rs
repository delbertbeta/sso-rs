use axum::extract::{Extension, Path};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::{
        application::ApplicationModel,
        application_secret::{ApplicationSecretModel, CreateSecretParams},
    },
    response::OkResponse,
};

#[derive(Serialize)]
struct ResponseApplicationSecret {
    id: i32,
    secret: String,
    created_at: String,
}

#[derive(Deserialize)]
pub struct GetSecretsListQueryParams {
    pub application_id: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    application_secret: ResponseApplicationSecret,
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

    let new_secret = application_secret_model
        .create_secret_by_application_id(CreateSecretParams {
            secret: uuid::Uuid::new_v4().to_string(),
            app_id: url_params.application_id,
            creator_id: user_id_from_session.user_id,
        })
        .await?;

    let res = SuccessResponse {
        application_secret: ResponseApplicationSecret {
            created_at: new_secret.created_at.to_string(),
            id: new_secret.id,
            secret: new_secret.secret,
        },
    };

    Ok(OkResponse::new(res))
}
