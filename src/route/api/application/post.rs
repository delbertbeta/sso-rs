use axum::{extract::Extension, Json};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{AppError, ServiceError},
    extractor::user_id_from_session::UserIdFromSession,
    model::{
        application::{ApplicationModel, CreateApplicationParams},
        image::ImageModel,
    },
    response::OkResponse,
};

#[derive(Serialize)]
pub struct SuccessResponse {}

#[derive(Deserialize, Validate)]
pub struct CreateApplicationPostParams {
    #[validate(
        required,
        length(min = 1, max = 24),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    name: Option<String>,

    #[validate(
        length(min = 1, max = 250),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    description: Option<String>,

    #[validate(required, url)]
    homepage_url: Option<String>,

    #[validate(required, url)]
    authorization_callback_url: Option<String>,

    #[validate(
        length(equal = 36),
        non_control_character,
        custom(function = "crate::util::validate_padding_string")
    )]
    icon_id: Option<String>,
}

pub async fn handler(
    Json(create_params): Json<CreateApplicationPostParams>,
    Extension(conn): Extension<DatabaseConnection>,
    user_id_from_session: UserIdFromSession,
) -> Result<OkResponse<SuccessResponse>, AppError> {
    create_params.validate()?;

    let name = create_params.name.unwrap();
    let icon_id = create_params.icon_id.unwrap();
    let homepage_url = create_params.homepage_url.unwrap();
    let authorization_callback_url = create_params.authorization_callback_url.unwrap();
    let description = create_params.description;

    let image_model = ImageModel::new(&conn);
    let icon = image_model.find_one_image_by_id(&icon_id).await?;

    if icon.is_none() || !(icon.unwrap().uploaded.map_or(false, |v| v == 1)) {
        return Err(ServiceError::ImageNotFound.into());
    }

    let application_model = ApplicationModel::new(&conn);

    let id = uuid::Uuid::new_v4().to_string();

    application_model
        .insert_application(CreateApplicationParams {
            id,
            name,
            icon_id,
            description,
            homepage_url,
            authorization_callback_url,
            creator_id: user_id_from_session.user_id,
        })
        .await?;

    Ok(OkResponse::new(SuccessResponse {}))
}
