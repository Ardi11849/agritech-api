use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::daily_readings::dto::{
    CreateDailyReadingRequest, DailyReadingListQuery, UnreportedTodayQuery,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/sessions/{session_id}/daily-readings",
    tag = "Farm Operations / Daily Readings",
    security(("BearerAuth" = [])),
    params(
        ("session_id" = String, Path, description = "Planting session ID"),
        DailyReadingListQuery
    ),
    responses(
        (status = 200, description = "Successfully retrieved daily readings for session", body = ApiResponse<SessionDailyReadingsResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Session not found", body = ApiResponse<()>)
    )
)]
pub async fn list_session_daily_readings(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(session_id): Path<String>,
    Query(query): Query<DailyReadingListQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state
        .daily_reading_service
        .get_daily_readings_by_session(&session_id, query)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved daily readings",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/sessions/{session_id}/daily-readings",
    tag = "Farm Operations / Daily Readings",
    security(("BearerAuth" = [])),
    params(
        ("session_id" = String, Path, description = "Planting session ID")
    ),
    request_body = CreateDailyReadingRequest,
    responses(
        (status = 201, description = "Successfully recorded daily reading", body = ApiResponse<DailyReadingItem>),
        (status = 400, description = "Bad request / session locked", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Session not found", body = ApiResponse<()>)
    )
)]
pub async fn create_session_daily_reading(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<String>,
    AppJson(req): AppJson<CreateDailyReadingRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state
        .daily_reading_service
        .create_daily_reading(&session_id, req, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully recorded daily reading",
        result,
    );
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/daily-readings/{id}",
    tag = "Farm Operations / Daily Readings",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Daily reading ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted daily reading", body = ApiResponse<()>),
        (status = 400, description = "Bad request / session locked", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 404, description = "Daily reading not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_daily_reading(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state
        .daily_reading_service
        .delete_daily_reading(&id)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted daily reading", ());
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/daily-readings/unreported-today",
    tag = "Farm Operations / Daily Readings",
    security(("BearerAuth" = [])),
    params(UnreportedTodayQuery),
    responses(
        (status = 200, description = "Successfully retrieved unreported active sessions", body = ApiResponse<UnreportedTodayResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn get_unreported_today(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<UnreportedTodayQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let result = state
        .daily_reading_service
        .get_unreported_today(query)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved unreported active sessions",
        result,
    );
    Ok((status, body))
}
