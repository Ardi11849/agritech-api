use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::common::response::ApiResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database Error: {0}")]
    Database(String),

    #[error("Timeout Error: {0}")]
    Timeout(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(msg) => {
                ApiResponse::error_response(StatusCode::NOT_FOUND, "Not Found", msg)
            }
            AppError::BadRequest(msg) => {
                ApiResponse::error_response(StatusCode::BAD_REQUEST, "Bad Request", msg)
            }
            AppError::Unauthorized(msg) => {
                ApiResponse::error_response(StatusCode::UNAUTHORIZED, "Unauthorized", msg)
            }
            AppError::Forbidden(msg) => {
                ApiResponse::error_response(StatusCode::FORBIDDEN, "Forbidden", msg)
            }
            AppError::Conflict(msg) => {
                ApiResponse::error_response(StatusCode::CONFLICT, "Conflict", msg)
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                ApiResponse::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service Unavailable",
                    "Database connection failed",
                )
            }
            AppError::Timeout(msg) => {
                tracing::error!("Timeout error: {}", msg);
                ApiResponse::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service Unavailable",
                    "Database connection failed",
                )
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                ApiResponse::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    msg,
                )
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err {
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return AppError::Conflict(db_err.message().to_string());
                }
                if code == "23503" {
                    let constraint = db_err.constraint().unwrap_or("");
                    let message = db_err.message();
                    if constraint == "planting_sessions_plant_type_id_fkey"
                        || message.contains("planting_sessions_plant_type_id_fkey")
                    {
                        return AppError::BadRequest(
                            "Tanaman tidak dapat dihapus karena masih digunakan pada sesi tanam."
                                .to_string(),
                        );
                    }
                    if constraint == "planting_sessions_installation_id_fkey"
                        || message.contains("planting_sessions_installation_id_fkey")
                    {
                        return AppError::BadRequest(
                            "Unit instalasi tidak dapat dihapus karena masih digunakan pada sesi tanam."
                                .to_string(),
                        );
                    }
                    return AppError::BadRequest(format!(
                        "Foreign key constraint violation: {}",
                        db_err.message()
                    ));
                }
                if code == "22003" {
                    return AppError::BadRequest(
                        "Nilai angka melebihi batas maksimum kapasitas sistem".to_string(),
                    );
                }
                if code == "22001" {
                    return AppError::BadRequest(
                        "Panjang teks melebihi batas maksimum karakter yang diizinkan".to_string(),
                    );
                }
            }
        }
        AppError::Database(err.to_string())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::BadRequest(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        let not_found = AppError::NotFound("item".into()).into_response();
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);

        let bad_req = AppError::BadRequest("bad input".into()).into_response();
        assert_eq!(bad_req.status(), StatusCode::BAD_REQUEST);

        let conflict = AppError::Conflict("already exists".into()).into_response();
        assert_eq!(conflict.status(), StatusCode::CONFLICT);

        let unauthorized = AppError::Unauthorized("invalid token".into()).into_response();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let forbidden = AppError::Forbidden("access denied".into()).into_response();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    }
}
