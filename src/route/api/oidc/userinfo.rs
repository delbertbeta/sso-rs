use axum::{
    extract::State,
    http::{HeaderMap},
    response::{IntoResponse, Json},
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde_json::{json};

use entity::{token, user};

use crate::{error::AppError, route::AppState, error::ServiceError};

pub async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            AppError::ServiceError(ServiceError::InvalidToken)
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::ServiceError(ServiceError::InvalidToken));
    }

    let access_token = &auth_header[7..];

    let token = token::Entity::find()
        .filter(token::Column::AccessToken.eq(access_token))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::ServiceError(ServiceError::InvalidToken))?;

    if token.expires_at < chrono::Utc::now().naive_utc() {
        return Err(AppError::ServiceError(ServiceError::InvalidToken));
    }

    let user = token.find_related(user::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::ServiceError(ServiceError::NotFound))?;

    let claims = json!({
        "sub": user.id,
        "name": user.username,
        "email": user.email,
        "picture": user.face_id,
    });

    Ok(Json(claims))
}
