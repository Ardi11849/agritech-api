use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ApiResponse<T = ()> {
    pub status: u16,
    pub message: String,
    #[schema(value_type = Option<Object>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(
        status: StatusCode,
        message: impl Into<String>,
        data: T,
    ) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            status,
            Json(ApiResponse {
                status: status.as_u16(),
                message: message.into(),
                data: Some(data),
                error: None,
            }),
        )
    }
}

impl ApiResponse<()> {
    pub fn error(
        status: StatusCode,
        message: impl Into<String>,
        error: impl Into<String>,
    ) -> (StatusCode, Json<ApiResponse<()>>) {
        (
            status,
            Json(ApiResponse {
                status: status.as_u16(),
                message: message.into(),
                data: None,
                error: Some(error.into()),
            }),
        )
    }

    pub fn error_response(
        status: StatusCode,
        message: impl Into<String>,
        error: impl Into<String>,
    ) -> Response {
        let (code, json_body) = Self::error(status, message, error);
        (code, json_body).into_response()
    }
}
