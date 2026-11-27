use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::masters::installations::dto::{
    CreateInstallationRequest, InstallationListQuery, UpdateInstallationRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/masters/installations",
    tag = "Master Data / Installations",
    security(("BearerAuth" = [])),
    params(InstallationListQuery),
    responses(
        (status = 200, description = "Successfully retrieved installations", body = ApiResponse<Vec<InstallationResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_installations(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<InstallationListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let installations = state.installation_service.list_installations(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved installations",
        installations,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/masters/installations/{id}",
    tag = "Master Data / Installations",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Installation UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved installation", body = ApiResponse<InstallationResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Installation not found", body = ApiResponse<()>)
    )
)]
pub async fn get_installation(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let installation = state.installation_service.get_installation(&id).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved installation",
        installation,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/masters/installations",
    tag = "Master Data / Installations",
    security(("BearerAuth" = [])),
    request_body = CreateInstallationRequest,
    responses(
        (status = 201, description = "Successfully created installation", body = ApiResponse<InstallationResponse>),
        (status = 400, description = "Bad Request - Validation Error or Invalid Location", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate installation name in location", body = ApiResponse<()>)
    )
)]
pub async fn create_installation(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreateInstallationRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let installation = state
        .installation_service
        .create_installation(payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created installation",
        installation,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/masters/installations/{id}",
    tag = "Master Data / Installations",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Installation UUID")
    ),
    request_body = UpdateInstallationRequest,
    responses(
        (status = 200, description = "Successfully updated installation", body = ApiResponse<InstallationResponse>),
        (status = 400, description = "Bad Request - Validation Error or Invalid Location", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Installation not found", body = ApiResponse<()>),
        (status = 409, description = "Conflict - Duplicate installation name in location", body = ApiResponse<()>)
    )
)]
pub async fn update_installation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateInstallationRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let installation = state
        .installation_service
        .update_installation(&id, payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully updated installation",
        installation,
    );
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/masters/installations/{id}",
    tag = "Master Data / Installations",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Installation UUID")
    ),
    responses(
        (status = 200, description = "Successfully deleted installation", body = ApiResponse<()>),
        (status = 400, description = "Bad Request - Foreign key dependency", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Installation not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_installation(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.installation_service.delete_installation(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted installation", ());
    Ok((status, body))
}
