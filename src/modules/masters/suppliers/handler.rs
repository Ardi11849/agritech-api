use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::suppliers::dto::{
    CreateSupplierRequest, SupplierListQuery, UpdateSupplierRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/suppliers",
    tag = "Master Data / Suppliers",
    security(("BearerAuth" = [])),
    params(SupplierListQuery),
    responses(
        (status = 200, description = "Successfully retrieved suppliers", body = ApiResponse<Vec<SupplierResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_suppliers(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SupplierListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let suppliers = state.supplier_service.list_suppliers(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved suppliers",
        suppliers,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/suppliers/{id}",
    tag = "Master Data / Suppliers",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Supplier UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved supplier", body = ApiResponse<SupplierResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Supplier not found", body = ApiResponse<()>)
    )
)]
pub async fn get_supplier(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let supplier = state.supplier_service.get_supplier(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved supplier", supplier);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/suppliers",
    tag = "Master Data / Suppliers",
    security(("BearerAuth" = [])),
    request_body = CreateSupplierRequest,
    responses(
        (status = 201, description = "Successfully created supplier", body = ApiResponse<SupplierResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict", body = ApiResponse<()>)
    )
)]
pub async fn create_supplier(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(req): AppJson<CreateSupplierRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let supplier = state
        .supplier_service
        .create_supplier(req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created supplier",
        supplier,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/suppliers/{id}",
    tag = "Master Data / Suppliers",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Supplier UUID")
    ),
    request_body = UpdateSupplierRequest,
    responses(
        (status = 200, description = "Successfully updated supplier", body = ApiResponse<SupplierResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Supplier not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict", body = ApiResponse<()>)
    )
)]
pub async fn update_supplier(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(req): AppJson<UpdateSupplierRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let supplier = state
        .supplier_service
        .update_supplier(&id, req, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated supplier", supplier);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/suppliers/{id}",
    tag = "Master Data / Suppliers",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Supplier UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted supplier", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Supplier not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_supplier(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.supplier_service.delete_supplier(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted supplier", ());
    Ok((status, body))
}
