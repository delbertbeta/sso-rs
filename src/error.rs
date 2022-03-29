use anyhow::Error as AnyError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use pbkdf2::password_hash::Error as PasswordError;
use sea_orm::DbErr;
use validator::ValidationErrors;

use crate::response::ErrorResponse;

#[derive(Debug)]
pub enum AppError {
    RegisterError(RegisterError),
    DatabaseError(DbErr),
    ValidationError(ValidationErrors),
    PasswordError(PasswordError),
    UnexpectedError(AnyError),
}

impl_from!(RegisterError, AppError, RegisterError);
impl_from!(DbErr, AppError, DatabaseError);
impl_from!(PasswordError, AppError, PasswordError);
impl_from!(ValidationErrors, AppError, ValidationError);
impl_from!(AnyError, AppError, UnexpectedError);

#[derive(Debug)]
pub enum RegisterError {
    DuplicatedUsername,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, error_message) = match self {
            AppError::RegisterError(RegisterError::DuplicatedUsername) => (
                StatusCode::BAD_REQUEST,
                100,
                "Username has been registered".to_string(),
            ),
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    200,
                    "Database error".to_string(),
                )
            }
            AppError::ValidationError(err) => {
                let message = format!("Input validation error: [{}]", err).replace('\n', ", ");
                (StatusCode::BAD_REQUEST, 300, message)
            }
            AppError::PasswordError(_) => {
                tracing::error!("Password error: {:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    400,
                    "Internal error".to_string(),
                )
            }
            AppError::UnexpectedError(err) => {
                tracing::error!("Unexpected error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    401,
                    "Internal error".to_string(),
                )
            }
        };

        tracing::warn!(
            "Response Error: code {:?}, error message: {:?}",
            code,
            error_message
        );

        let body = Json(ErrorResponse::new(code, error_message.to_string()));

        (status, body).into_response()
    }
}
