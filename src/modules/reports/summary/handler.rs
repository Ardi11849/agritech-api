use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::error::AppError;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::reports::summary::dto::{CashflowSummaryQuery, CashflowSummaryResponse};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/reports/summary",
    tag = "Reports, Audit Trail & AI Intelligence",
    security(("BearerAuth" = [])),
    params(CashflowSummaryQuery),
    responses(
        (status = 200, description = "Laporan ringkasan arus kas dan buku besar keuangan", body = ApiResponse<CashflowSummaryResponse>),
        (status = 400, description = "Bad Request - filter tanggal tidak valid", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_cashflow_summary(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<CashflowSummaryQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .report_summary_service
        .get_cashflow_summary(&query)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully fetched cashflow summary report",
        res,
    );
    Ok((status, body))
}
