use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::error::AppError;
use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::finance::sales::dto::{
    CreateSaleRequest, SaleListResponse, SaleQuery, SaleResponse, UpdateSaleRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/finance/sales",
    tag = "Finance / Sales",
    security(("BearerAuth" = [])),
    params(SaleQuery),
    responses(
        (status = 200, description = "Daftar transaksi penjualan", body = ApiResponse<SaleListResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_sales(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SaleQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.sale_service.get_sales(&query).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully retrieved sales", res);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/sales/{id}",
    tag = "Finance / Sales",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Sale ID")
    ),
    responses(
        (status = 200, description = "Detail transaksi penjualan", body = ApiResponse<SaleResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn get_sale(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.sale_service.get_sale_by_id(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved sale detail", res);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/sales",
    tag = "Finance / Sales",
    security(("BearerAuth" = [])),
    request_body = CreateSaleRequest,
    responses(
        (status = 201, description = "Transaksi penjualan berhasil dicatat", body = ApiResponse<SaleResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn create_sale(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreateSaleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .sale_service
        .create_sale(payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Transaksi penjualan berhasil dicatat",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/finance/sales/{id}",
    tag = "Finance / Sales",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Sale ID")
    ),
    request_body = UpdateSaleRequest,
    responses(
        (status = 200, description = "Transaksi penjualan berhasil diperbarui", body = ApiResponse<SaleResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn update_sale(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateSaleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .sale_service
        .update_sale(&id, payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Transaksi penjualan berhasil diperbarui",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/finance/sales/{id}",
    tag = "Finance / Sales",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Sale ID")
    ),
    responses(
        (status = 200, description = "Transaksi penjualan berhasil dihapus", body = ApiResponse<()>),
        (status = 400, description = "Bad Request - sesi tanam terkait sudah terkunci", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn delete_sale(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.sale_service.delete_sale(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted sale", ());
    Ok((status, body))
}
