use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::sessions::dto::{
    CompleteHarvestRequest, CreateSessionRequest, SessionListQuery, UpdateSessionRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/sessions",
    tag = "Farm Operations / Planting Sessions",
    security(("BearerAuth" = [])),
    params(SessionListQuery),
    responses(
        (status = 200, description = "Successfully retrieved planting sessions", body = ApiResponse<SessionListResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SessionListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state.session_service.get_sessions(query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved planting sessions",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/sessions/{id}",
    tag = "Farm Operations / Planting Sessions",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Planting session ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved planting session detail", body = ApiResponse<SessionDetailResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Session not found", body = ApiResponse<()>)
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state.session_service.get_session_by_id(&id, None).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved planting session detail",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions",
    tag = "Farm Operations / Planting Sessions",
    security(("BearerAuth" = [])),
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Successfully created planting session", body = ApiResponse<SessionDetailResponse>),
        (status = 400, description = "Bad request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn create_session(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(req): AppJson<CreateSessionRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state
        .session_service
        .create_session(req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created planting session",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/sessions/{id}",
    tag = "Farm Operations / Planting Sessions",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Planting session ID")
    ),
    request_body = UpdateSessionRequest,
    responses(
        (status = 200, description = "Successfully updated planting session", body = ApiResponse<SessionDetailResponse>),
        (status = 400, description = "Bad request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Session not found", body = ApiResponse<()>)
    )
)]
pub async fn update_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(req): AppJson<UpdateSessionRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state
        .session_service
        .update_session(&id, req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully updated planting session",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions/{id}/complete-harvest",
    tag = "Farm Operations / Planting Sessions",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Planting session ID")
    ),
    request_body = CompleteHarvestRequest,
    responses(
        (status = 200, description = "Successfully completed harvest and locked session", body = ApiResponse<SessionDetailResponse>),
        (status = 400, description = "Bad request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Session not found", body = ApiResponse<()>)
    )
)]
pub async fn complete_harvest(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(req): AppJson<CompleteHarvestRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state
        .session_service
        .complete_harvest(&id, req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully completed harvest and locked session",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/sessions/{id}",
    tag = "Farm Operations / Planting Sessions",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Planting session ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted planting session", body = ApiResponse<()>),
        (status = 400, description = "Bad request / session locked", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Session not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_session(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.session_service.delete_session(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted planting session", ());
    Ok((status, body))
}
