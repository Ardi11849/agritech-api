use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::error::AppError;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::reports::audit::dto::{AuditLogListResponse, AuditLogQuery};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/reports/audit",
    tag = "Reports, Audit Trail & AI Intelligence",
    security(("BearerAuth" = [])),
    params(AuditLogQuery),
    responses(
        (status = 200, description = "Daftar jejak audit aktivitas sistem dan perubahan data", body = ApiResponse<AuditLogListResponse>),
        (status = 400, description = "Bad Request - parameter filter tidak valid", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_logs(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.audit_service.get_logs(&query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully fetched audit trail logs", res);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/audit/modules",
    tag = "Reports, Audit Trail & AI Intelligence",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Daftar modul sistem unik yang memiliki riwayat audit", body = ApiResponse<Vec<String>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_modules(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let res = state.audit_service.get_modules().await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully fetched audit modules", res);
    Ok((status, body))
}
