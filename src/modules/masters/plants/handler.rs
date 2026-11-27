use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::plants::dto::{
    CreatePlantRequest, PlantListQuery, UpdatePlantRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/plants",
    tag = "Master Data / Plants",
    security(("BearerAuth" = [])),
    params(PlantListQuery),
    responses(
        (status = 200, description = "Successfully retrieved plants", body = ApiResponse<Vec<PlantResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_plants(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<PlantListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let plants = state.plant_service.list_plants(query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved plants", plants);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/plants/{id}",
    tag = "Master Data / Plants",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Plant Type UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved plant", body = ApiResponse<PlantResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Plant not found", body = ApiResponse<()>)
    )
)]
pub async fn get_plant(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let plant = state.plant_service.get_plant(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved plant", plant);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/plants",
    tag = "Master Data / Plants",
    security(("BearerAuth" = [])),
    request_body = CreatePlantRequest,
    responses(
        (status = 201, description = "Successfully created plant", body = ApiResponse<PlantResponse>),
        (status = 400, description = "Bad Request - Validation or Boundary Error", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate plant name and variety", body = ApiResponse<()>)
    )
)]
pub async fn create_plant(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreatePlantRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let plant = state
        .plant_service
        .create_plant(payload, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created plant", plant);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/plants/{id}",
    tag = "Master Data / Plants",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Plant Type UUID")
    ),
    request_body = UpdatePlantRequest,
    responses(
        (status = 200, description = "Successfully updated plant", body = ApiResponse<PlantResponse>),
        (status = 400, description = "Bad Request - Validation Error", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Plant not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate plant name and variety", body = ApiResponse<()>)
    )
)]
pub async fn update_plant(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdatePlantRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let plant = state
        .plant_service
        .update_plant(&id, payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully updated plant", plant);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/plants/{id}",
    tag = "Master Data / Plants",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Plant Type UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted plant", body = ApiResponse<()>),
        (status = 400, description = "Bad Request - Foreign key dependency", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Plant not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_plant(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.plant_service.delete_plant(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted plant", ());
    Ok((status, body))
}
