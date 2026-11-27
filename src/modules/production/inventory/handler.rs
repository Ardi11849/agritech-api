use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::inventory::dto::{
    CreateMovementRequest, CreateStockOpnameRequest, InventorySummaryQuery, StockCardQuery,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/inventory/summary",
    tag = "Inventory / Warehouse Ledger",
    security(("BearerAuth" = [])),
    params(InventorySummaryQuery),
    responses(
        (status = 200, description = "Successfully retrieved inventory summary", body = ApiResponse<InventorySummaryResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn get_summary(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<InventorySummaryQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let summary = state.inventory_service.get_summary(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved inventory summary",
        summary,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/movements",
    tag = "Inventory / Warehouse Ledger",
    security(("BearerAuth" = [])),
    request_body = CreateMovementRequest,
    responses(
        (status = 201, description = "Successfully recorded inventory movement", body = ApiResponse<InventoryLedgerResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item not found", body = ApiResponse<()>)
    )
)]
pub async fn record_movement(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(req): AppJson<CreateMovementRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let ledger = state
        .inventory_service
        .record_movement(req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully recorded inventory movement",
        ledger,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/stock-opname",
    tag = "Inventory / Warehouse Ledger",
    security(("BearerAuth" = [])),
    request_body = CreateStockOpnameRequest,
    responses(
        (status = 201, description = "Successfully recorded stock opname", body = ApiResponse<InventoryLedgerResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item not found", body = ApiResponse<()>)
    )
)]
pub async fn record_stock_opname(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(req): AppJson<CreateStockOpnameRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let ledger = state
        .inventory_service
        .record_stock_opname(req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully recorded stock opname",
        ledger,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/items/{id}/stock-card",
    tag = "Inventory / Warehouse Ledger",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Master Item UUID"),
        StockCardQuery
    ),
    responses(
        (status = 200, description = "Successfully retrieved stock card", body = ApiResponse<StockCardResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item not found", body = ApiResponse<()>)
    )
)]
pub async fn get_stock_card(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
    Query(query): Query<StockCardQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let stock_card = state.inventory_service.get_stock_card(&id, query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved stock card",
        stock_card,
    );
    Ok((status, body))
}
