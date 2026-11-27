use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::error::AppError;
use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::finance::purchases::dto::{
    CreatePurchaseRequest, PurchaseListResponse, PurchaseQuery, PurchaseResponse,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/finance/purchases",
    tag = "Finance / Purchases",
    security(("BearerAuth" = [])),
    params(PurchaseQuery),
    responses(
        (status = 200, description = "Daftar nota pembelian", body = ApiResponse<PurchaseListResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_purchases(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<PurchaseQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.purchase_service.get_purchases(&query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved purchases", res);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/purchases/{id}",
    tag = "Finance / Purchases",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Purchase ID")
    ),
    responses(
        (status = 200, description = "Detail nota pembelian", body = ApiResponse<PurchaseResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn get_purchase(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.purchase_service.get_purchase_by_id(&id).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved purchase detail",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/purchases",
    tag = "Finance / Purchases",
    security(("BearerAuth" = [])),
    request_body = CreatePurchaseRequest,
    responses(
        (status = 201, description = "Nota pembelian berhasil dibuat", body = ApiResponse<PurchaseResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn create_purchase(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreatePurchaseRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .purchase_service
        .create_purchase(payload, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Nota pembelian berhasil dibuat", res);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/finance/purchases/{id}",
    tag = "Finance / Purchases",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Purchase ID")
    ),
    responses(
        (status = 200, description = "Nota pembelian berhasil dihapus", body = ApiResponse<()>),
        (status = 400, description = "Bad Request - stok barang sudah terpakai atau sesi terkunci", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn delete_purchase(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state
        .purchase_service
        .delete_purchase(&id, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully deleted purchase and reversed synced inventory",
        (),
    );
    Ok((status, body))
}
