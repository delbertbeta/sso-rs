use anyhow::Error as AnyError;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use openssl::error::ErrorStack;
use pbkdf2::password_hash::Error as PasswordError;
use qcloud::error::QCloudError;
use sea_orm::DbErr;
use validator::ValidationErrors;
use volo_grpc::Status;

use crate::response::ErrorResponse;

#[derive(Debug)]
pub enum AppError {
    ServiceError(ServiceError),
    DatabaseError(DbErr),
    ValidationError(ValidationErrors),
    PasswordError(PasswordError),
    RsaError(ErrorStack),
    UnexpectedError(AnyError),
    JSONError(JsonRejection),
    QCloudError(QCloudError),
}

impl_from!(ServiceError, AppError, ServiceError);
impl_from!(DbErr, AppError, DatabaseError);
impl_from!(PasswordError, AppError, PasswordError);
impl_from!(ValidationErrors, AppError, ValidationError);
impl_from!(JsonRejection, AppError, JSONError);
impl_from!(AnyError, AppError, UnexpectedError);
impl_from!(ErrorStack, AppError, RsaError);
impl_from!(QCloudError, AppError, QCloudError);

#[derive(Debug)]
pub enum ServiceError {
    DuplicatedUsername,
    InvalidRsaToken,
    DecryptPasswordError,
    InvalidPasswordLength,
    LoginFailed,
    LoginRequired,
    NotFound,
    PermissionDenied,
    ImageNotFound,
}

struct ErrorResponseInfo {
    status: StatusCode,
    code: i32,
    error_message: String,
}

impl AppError {
    fn get_error_response_info(self) -> ErrorResponseInfo {
        let (status, code, error_message) = match self {
            AppError::ServiceError(ServiceError::DuplicatedUsername) => (
                StatusCode::BAD_REQUEST,
                100,
                "Username has been registered".to_string(),
            ),
            AppError::ServiceError(ServiceError::InvalidRsaToken) => (
                StatusCode::BAD_REQUEST,
                101,
                "Invalid Rsa token".to_string(),
            ),
            AppError::ServiceError(ServiceError::DecryptPasswordError) => (
                StatusCode::BAD_REQUEST,
                102,
                "Failed to decrypt password".to_string(),
            ),
            AppError::ServiceError(ServiceError::InvalidPasswordLength) => (
                StatusCode::BAD_REQUEST,
                103,
                "Password format is invalid".to_string(),
            ),
            AppError::ServiceError(ServiceError::LoginFailed) => {
                (StatusCode::BAD_REQUEST, 104, "Login failed".to_string())
            }
            AppError::ServiceError(ServiceError::LoginRequired) => {
                let message = format!("Login required");
                (StatusCode::UNAUTHORIZED, 106, message)
            }
            AppError::ServiceError(ServiceError::NotFound) => {
                let message = format!("Record not found");
                (StatusCode::NOT_FOUND, 107, message)
            }
            AppError::ServiceError(ServiceError::ImageNotFound) => {
                let message = format!("Image not found");
                (StatusCode::BAD_REQUEST, 108, message)
            }
            AppError::ValidationError(err) => {
                let message = format!("Input validation error: [{}]", err).replace('\n', ", ");
                (StatusCode::BAD_REQUEST, 105, message)
            }
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    501,
                    "Database error".to_string(),
                )
            }
            AppError::QCloudError(err) => {
                tracing::error!("QCloud error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    300,
                    "QCloud error".to_string(),
                )
            }
            AppError::ServiceError(ServiceError::PermissionDenied) => {
                (StatusCode::FORBIDDEN, 403, "Permission Denied".to_string())
            }
            AppError::PasswordError(_) => {
                tracing::error!("Password error: {:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    502,
                    "Internal error".to_string(),
                )
            }
            AppError::RsaError(err) => {
                tracing::error!("Rsa crypto error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    503,
                    "Crypto error".to_string(),
                )
            }
            AppError::JSONError(err) => {
                tracing::warn!("JSON parse error: {:?}", err);
                (StatusCode::BAD_REQUEST, 503, "Invalid input".to_string())
            }
            AppError::UnexpectedError(err) => {
                tracing::error!("Unexpected error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    500,
                    "Internal error".to_string(),
                )
            }
        };
        ErrorResponseInfo {
            error_message,
            code,
            status,
        }
    }
}

impl From<AppError> for Status {
    fn from(error: AppError) -> Self {
        let ErrorResponseInfo {
            code,
            error_message,
            status,
        } = error.get_error_response_info();

        Status::internal(error_message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let ErrorResponseInfo {
            code,
            error_message,
            status,
        } = self.get_error_response_info();

        tracing::warn!(
            "Response Error: code {:?}, error message: {:?}",
            code,
            error_message
        );

        let body = Json(ErrorResponse::new(code as usize, error_message.to_string()));

        (status, body).into_response()
    }
}
