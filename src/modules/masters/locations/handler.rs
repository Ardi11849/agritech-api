use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::locations::dto::{
    CreateLocationRequest, LocationListQuery, UpdateLocationRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/locations",
    tag = "Master Data / Locations",
    security(("BearerAuth" = [])),
    params(LocationListQuery),
    responses(
        (status = 200, description = "Successfully retrieved locations", body = ApiResponse<Vec<LocationResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_locations(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<LocationListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let locations = state.location_service.list_locations(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved locations",
        locations,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/locations/{id}",
    tag = "Master Data / Locations",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Location UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved location", body = ApiResponse<LocationResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Location not found", body = ApiResponse<()>)
    )
)]
pub async fn get_location(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let location = state.location_service.get_location(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved location", location);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/locations",
    tag = "Master Data / Locations",
    security(("BearerAuth" = [])),
    request_body = CreateLocationRequest,
    responses(
        (status = 201, description = "Successfully created location", body = ApiResponse<LocationResponse>),
        (status = 400, description = "Bad Request - Validation Error", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate location name", body = ApiResponse<()>)
    )
)]
pub async fn create_location(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreateLocationRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let location = state
        .location_service
        .create_location(payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created location",
        location,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/locations/{id}",
    tag = "Master Data / Locations",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Location UUID")
    ),
    request_body = UpdateLocationRequest,
    responses(
        (status = 200, description = "Successfully updated location", body = ApiResponse<LocationResponse>),
        (status = 400, description = "Bad Request - Validation Error", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Location not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate location name", body = ApiResponse<()>)
    )
)]
pub async fn update_location(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateLocationRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let location = state
        .location_service
        .update_location(&id, payload, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated location", location);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/locations/{id}",
    tag = "Master Data / Locations",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Location UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted location", body = ApiResponse<()>),
        (status = 400, description = "Bad Request - Location has active installations", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Location not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_location(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.location_service.delete_location(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted location", ());
    Ok((status, body))
}
