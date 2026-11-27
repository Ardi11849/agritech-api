use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::error::AppError;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::reports::ai_audit::dto::{AiAuditScanQuery, AiAuditScanResponse};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/reports/ai-audit/scan",
    tag = "Reports, Audit Trail & AI Intelligence",
    security(("BearerAuth" = [])),
    params(AiAuditScanQuery),
    responses(
        (status = 200, description = "Pemindaian anomali operasional dan keuangan oleh AI Audit", body = ApiResponse<AiAuditScanResponse>),
        (status = 400, description = "Bad Request - kategori anomali tidak valid", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn scan(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<AiAuditScanQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.ai_audit_service.scan(&query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "AI audit scan completed successfully", res);
    Ok((status, body))
}
