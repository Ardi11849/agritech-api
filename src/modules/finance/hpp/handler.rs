use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::common::error::AppError;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::finance::hpp::dto::{HppListQuery, HppListResponse, RHPPDetailResponse};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct DateQuery {
    pub date: Option<NaiveDate>,
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/hpp",
    tag = "Finance / HPP",
    security(("BearerAuth" = [])),
    params(HppListQuery),
    responses(
        (status = 200, description = "Daftar analisa realisasi HPP dan profitabilitas seluruh sesi", body = ApiResponse<HppListResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn get_hpp_list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<HppListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.hpp_service.get_hpp_list(&query).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved HPP profitability analysis",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/hpp/{session_id}",
    tag = "Finance / HPP",
    security(("BearerAuth" = [])),
    params(
        ("session_id" = String, Path, description = "Planting Session ID"),
        ("date" = Option<NaiveDate>, Query, description = "Evaluation date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "Rincian laporan RHPP lengkap sesi tanam", body = ApiResponse<RHPPDetailResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn get_session_rhpp(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(session_id): Path<String>,
    Query(q): Query<DateQuery>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .hpp_service
        .get_session_rhpp(&session_id, q.date)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved session RHPP report",
        res,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/hpp/{session_id}/lock-snapshot",
    tag = "Finance / HPP",
    security(("BearerAuth" = [])),
    params(
        ("session_id" = String, Path, description = "Planting Session ID")
    ),
    responses(
        (status = 200, description = "Snapshot angka finansial sesi berhasil dikunci permanen", body = ApiResponse<RHPPDetailResponse>),
        (status = 400, description = "Bad Request - sesi belum selesai atau sudah pernah dikunci", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn lock_snapshot(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let res = state
        .hpp_service
        .lock_session_snapshot(&session_id, &auth.username)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Snapshot finansial sesi berhasil dikunci permanen",
        res,
    );
    Ok((status, body))
}
