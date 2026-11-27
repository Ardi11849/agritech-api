use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::response::ApiResponse;
use crate::modules::health::dto::ComponentStatus;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "System / Health",
    responses(
        (status = 200, description = "Service is healthy", body = ApiResponse<HealthStatus>),
        (status = 500, description = "Service Unavailable - Database connection failed", body = ApiResponse<()>)
    )
)]
pub async fn check_health(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.health_service.check_health().await;

    if status.database == ComponentStatus::Down {
        ApiResponse::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Service Unavailable",
            "Database connection failed",
        )
    } else {
        let (code, json_body) = ApiResponse::success(StatusCode::OK, "Service is healthy", status);
        (code, json_body).into_response()
    }
}
