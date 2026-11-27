use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::items::dto::{
    CreateMasterItemRequest, ItemListQuery, UpdateMasterItemRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/items",
    tag = "Master Data / Items",
    security(("BearerAuth" = [])),
    params(ItemListQuery),
    responses(
        (status = 200, description = "Successfully retrieved items", body = ApiResponse<Vec<MasterItemResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_items(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ItemListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let items = state.item_service.list_items(query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved items", items);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/items/{id}",
    tag = "Master Data / Items",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Item UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved item", body = ApiResponse<MasterItemResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item not found", body = ApiResponse<()>)
    )
)]
pub async fn get_item(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let item = state.item_service.get_item(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully retrieved item", item);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/items",
    tag = "Master Data / Items",
    security(("BearerAuth" = [])),
    request_body = CreateMasterItemRequest,
    responses(
        (status = 201, description = "Successfully created item", body = ApiResponse<MasterItemResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict", body = ApiResponse<()>)
    )
)]
pub async fn create_item(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(req): AppJson<CreateMasterItemRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let item = state.item_service.create_item(req, &auth.username).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created item", item);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/items/{id}",
    tag = "Master Data / Items",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Item UUID")
    ),
    request_body = UpdateMasterItemRequest,
    responses(
        (status = 200, description = "Successfully updated item", body = ApiResponse<MasterItemResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict", body = ApiResponse<()>)
    )
)]
pub async fn update_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(req): AppJson<UpdateMasterItemRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let item = state
        .item_service
        .update_item(&id, req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully updated item", item);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/items/{id}",
    tag = "Master Data / Items",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Item UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted item", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_item(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.item_service.delete_item(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted item", ());
    Ok((status, body))
}
