use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::error::AppError;
use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::finance::tax::dto::{
    PayTaxRequest, TaxDashboardData, TaxDashboardQuery, TaxReportItem, TaxSettingsDto,
    UpdateTaxSettingsRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/finance/tax/dashboard",
    tag = "Finance / Tax",
    security(("BearerAuth" = [])),
    params(TaxDashboardQuery),
    responses(
        (status = 200, description = "Dashboard perhitungan pajak UMKM dan riwayat", body = ApiResponse<TaxDashboardData>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_tax_dashboard(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<TaxDashboardQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.tax_service.get_dashboard(&query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved tax dashboard", res);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/tax/pay",
    tag = "Finance / Tax",
    security(("BearerAuth" = [])),
    request_body = PayTaxRequest,
    responses(
        (status = 200, description = "Pelunasan pajak bulanan berhasil dicatat", body = ApiResponse<TaxReportItem>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn pay_tax(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<PayTaxRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.tax_service.pay_tax(payload, &auth.username).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Pelunasan pajak bulanan berhasil dicatat",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/finance/tax/settings",
    tag = "Finance / Tax",
    security(("BearerAuth" = [])),
    request_body = UpdateTaxSettingsRequest,
    responses(
        (status = 200, description = "Pengaturan tarif pajak berhasil diperbarui", body = ApiResponse<TaxSettingsDto>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn update_tax_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<UpdateTaxSettingsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .tax_service
        .update_settings(payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Pengaturan tarif pajak berhasil diperbarui",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/finance/tax/reports/{id}",
    tag = "Finance / Tax",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Tax Report ID")
    ),
    responses(
        (status = 200, description = "Riwayat laporan pajak berhasil dihapus", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn delete_tax_report(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.tax_service.delete_report(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted tax report", ());
    Ok((status, body))
}
