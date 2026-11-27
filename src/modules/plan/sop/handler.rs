use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::plan::sop::dto::{
    CreateSopRequest, SopListResponse, SopQuery, SopResponse, UpdateSopRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/plan/sops",
    tag = "Plan / SOP",
    security(("BearerAuth" = [])),
    params(SopQuery),
    responses(
        (status = 200, description = "Daftar SOP beserta KPI", body = ApiResponse<SopListResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn list_sops(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<SopQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state.sop_service.get_sops(&query).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully retrieved SOPs", res);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/plan/sops/{id}",
    tag = "Plan / SOP",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "SOP ID")
    ),
    responses(
        (status = 200, description = "Detail SOP", body = ApiResponse<SopResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn get_sop(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state.sop_service.get_sop_by_id(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved SOP detail", res);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/plan/sops",
    tag = "Plan / SOP",
    security(("BearerAuth" = [])),
    request_body = CreateSopRequest,
    responses(
        (status = 201, description = "SOP berhasil dibuat", body = ApiResponse<SopResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn create_sop(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreateSopRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state
        .sop_service
        .create_sop(payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(StatusCode::CREATED, "Successfully created SOP", res);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/plan/sops/{id}",
    tag = "Plan / SOP",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "SOP ID")
    ),
    request_body = UpdateSopRequest,
    responses(
        (status = 200, description = "SOP berhasil diperbarui", body = ApiResponse<SopResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn update_sop(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateSopRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state
        .sop_service
        .update_sop(&id, payload, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully updated SOP", res);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/plan/sops/{id}",
    tag = "Plan / SOP",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "SOP ID")
    ),
    responses(
        (status = 200, description = "SOP berhasil dihapus", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn delete_sop(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.sop_service.delete_sop(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted SOP", ());
    Ok((status, body))
}
