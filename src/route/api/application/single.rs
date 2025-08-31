use axum::extract::{Extension, Path};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    constants::ENVS,
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
    name: String,
    icon_url: Option<String>,
    id: String,
    description: Option<String>,
    homepage_url: String,
    redirect_uris: String,
    grant_types: String,
}

pub async fn handler(
    Extension(conn): Extension<DatabaseConnection>,
    Path(url_params): Path<GetSecretsListQueryParams>,
    user_id_from_session: UserIdFromSession,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let application_model = ApplicationModel::new(&conn);
    let application_secret_model = ApplicationSecretModel::new(&conn);

    let (application, icon) = application_model
        .find_one_application_by_id(url_params.application_id.as_str())
        .await?
        .ok_or(ServiceError::NotFound)?;

    if application.creator_id != user_id_from_session.user_id {
        return Err(ServiceError::PermissionDenied.into());
    }

    let icon_url = icon.and_then(|f| Some(format!("{}{}", ENVS.cdn_base_url, f.path)));

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
        name: application.name,
        icon_url,
        id: application.id,
        description: application.description,
        homepage_url: application.homepage_url,
        redirect_uris: application.redirect_uris,
        grant_types: application.grant_types,
    };

    Ok(OkResponse::new(res))
}
