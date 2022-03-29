use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OkResponse<T>
where
    T: Serialize,
{
    pub code: usize,
    pub data: T,
}

impl<T> OkResponse<T>
where
    T: Serialize,
{
    pub fn new(data: T) -> Self {
        Self { code: 0, data }
    }
}

impl<T> IntoResponse for OkResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let body = Json(self);

        (StatusCode::OK, body).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: usize,
    pub msg: String,
}

impl ErrorResponse {
    pub fn new(code: usize, msg: String) -> Self {
        ErrorResponse { code, msg }
    }
}
