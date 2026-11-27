use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::item_categories::dto::{
    CreateItemCategoryRequest, ItemCategoryListQuery, UpdateItemCategoryRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/item-categories",
    tag = "Master Data / Item Categories",
    security(("BearerAuth" = [])),
    params(ItemCategoryListQuery),
    responses(
        (status = 200, description = "Successfully retrieved item categories", body = ApiResponse<Vec<ItemCategoryResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_categories(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ItemCategoryListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let categories = state.item_category_service.list_categories(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved item categories",
        categories,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/item-categories/{id}",
    tag = "Master Data / Item Categories",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Item Category UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved item category", body = ApiResponse<ItemCategoryResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item category not found", body = ApiResponse<()>)
    )
)]
pub async fn get_category(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let category = state.item_category_service.get_category(&id).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved item category",
        category,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/item-categories",
    tag = "Master Data / Item Categories",
    security(("BearerAuth" = [])),
    request_body = CreateItemCategoryRequest,
    responses(
        (status = 201, description = "Successfully created item category", body = ApiResponse<ItemCategoryResponse>),
        (status = 400, description = "Bad Request - Validation Error", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate item category name", body = ApiResponse<()>)
    )
)]
pub async fn create_category(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreateItemCategoryRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let category = state
        .item_category_service
        .create_category(payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created item category",
        category,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/item-categories/{id}",
    tag = "Master Data / Item Categories",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Item Category UUID")
    ),
    request_body = UpdateItemCategoryRequest,
    responses(
        (status = 200, description = "Successfully updated item category", body = ApiResponse<ItemCategoryResponse>),
        (status = 400, description = "Bad Request - Validation Error", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item category not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate item category name", body = ApiResponse<()>)
    )
)]
pub async fn update_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateItemCategoryRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let category = state
        .item_category_service
        .update_category(&id, payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully updated item category",
        category,
    );
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/item-categories/{id}",
    tag = "Master Data / Item Categories",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Item Category UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted item category", body = ApiResponse<()>),
        (status = 400, description = "Bad Request - Category still contains items", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Item category not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_category(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.item_category_service.delete_category(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted item category", ());
    Ok((status, body))
}
