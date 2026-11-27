use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::auth::dto::{LoginRequest, RegisterRequest};
use crate::AppState;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Authentication & Identity",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Successfully registered user", body = ApiResponse<RegisterResponse>),
        (status = 400, description = "Validation error or username/email already exists", body = ApiResponse<()>),
        (status = 401, description = "Missing or invalid authentication token", body = ApiResponse<()>),
        (status = 403, description = "Forbidden: Insufficient privileges", body = ApiResponse<()>)
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    )
)]
pub async fn register(
    State(state): State<AppState>,
    _auth: AuthUser,
    AppJson(payload): AppJson<RegisterRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state.auth_service.register(payload).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully registered user", result);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Authentication & Identity",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Successfully logged in", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Invalid credentials", body = ApiResponse<()>)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    AppJson(payload): AppJson<LoginRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state.auth_service.login(payload).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully logged in", result);
    Ok((status, body))
}
