use axum::extract::Extension;
use sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::{
    constants::ENVS, error::AppError, extractor::user_id_from_session::UserIdFromSession,
    model::application::ApplicationModel, response::OkResponse,
};

#[derive(Serialize)]
struct ResponseApplication {
    name: String,
    icon_url: Option<String>,
    id: String,
    description: String,
    homepage_url: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    applications: Vec<ResponseApplication>,
}

pub async fn handler(
    Extension(conn): Extension<DatabaseConnection>,
    user_id_from_session: UserIdFromSession,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    let application_model = ApplicationModel::new(&conn);

    let applications = application_model
        .find_applications_by_user_id(&user_id_from_session.user_id)
        .await?;

    let res = SuccessResponse {
        applications: applications
            .into_iter()
            .map(|(app, image)| ResponseApplication {
                name: app.name,
                id: app.id,
                homepage_url: app.homepage_url,
                description: app.description.unwrap_or("".to_string()),
                icon_url: image.map_or(None, |f| Some(format!("{}{}", ENVS.cdn_base_url, f.path))),
            })
            .collect(),
    };

    Ok(OkResponse::new(res))
}
