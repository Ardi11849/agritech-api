use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::auth::dto::{LoginRequest, RegisterRequest};
use crate::modules::reports::audit::dto::AuditAction;
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
    headers: HeaderMap,
    AppJson(payload): AppJson<LoginRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state.auth_service.login(payload).await?;

    // Extract client IP safely (X-Forwarded-For segment 1 -> X-Real-IP -> None)
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .map(|s| if s.len() > 45 { s[..45].to_string() } else { s });

    // Record non-blocking audit log event for successful LOGIN
    let audit_username = result.user.username.clone();
    let audit_user_id = Some(result.user.id.clone());
    let audit_desc = format!("Pengguna '{}' berhasil masuk ke sistem", audit_username);
    let audit_new_data = Some(serde_json::json!({
        "username": result.user.username,
        "role_id": result.user.role_id,
        "name": result.user.name
    }));

    if let Err(e) = state
        .audit_service
        .record_event(
            AuditAction::LOGIN,
            "AUTH",
            audit_user_id,
            &audit_username,
            &result.user.id,
            &audit_desc,
            None,
            audit_new_data,
            client_ip,
        )
        .await
    {
        tracing::warn!("Failed to record audit log for login event: {}", e);
    }

    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully logged in", result);
    Ok((status, body))
}
