use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
#[allow(unused_imports)]
use crate::modules::plan::tasks::dto::{
    CreateTaskRequest, TaskListResponse, TaskQuery, TaskResponse, UpdateTaskRequest,
    UpdateTaskStatusRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/plan/tasks",
    tag = "Plan / Tasks",
    security(("BearerAuth" = [])),
    params(TaskQuery),
    responses(
        (status = 200, description = "Daftar Tugas Kanban beserta KPI", body = ApiResponse<TaskListResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn list_tasks(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<TaskQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state.task_service.get_tasks(&query).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved farm tasks", res);
    Ok((status, body))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TaskDetailQuery {
    pub date: Option<chrono::NaiveDate>,
}

#[utoipa::path(
    get,
    path = "/api/v1/plan/tasks/{id}",
    tag = "Plan / Tasks",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Task ID"),
        TaskDetailQuery
    ),
    responses(
        (status = 200, description = "Detail Tugas", body = ApiResponse<TaskResponse>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn get_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
    Query(query): Query<TaskDetailQuery>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state.task_service.get_task_by_id(&id, query.date).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved task detail", res);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/plan/tasks",
    tag = "Plan / Tasks",
    security(("BearerAuth" = [])),
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Tugas berhasil dibuat", body = ApiResponse<TaskResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>)
    )
)]
pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    AppJson(payload): AppJson<CreateTaskRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state
        .task_service
        .create_task(payload, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created farm task", res);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/plan/tasks/{id}",
    tag = "Plan / Tasks",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Tugas berhasil diperbarui", body = ApiResponse<TaskResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateTaskRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state
        .task_service
        .update_task(&id, payload, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated farm task", res);
    Ok((status, body))
}

#[utoipa::path(
    patch,
    path = "/api/v1/plan/tasks/{id}/status",
    tag = "Plan / Tasks",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    request_body = UpdateTaskStatusRequest,
    responses(
        (status = 200, description = "Status tugas berhasil diperbarui", body = ApiResponse<TaskResponse>),
        (status = 400, description = "Bad Request", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn update_task_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    AppJson(payload): AppJson<UpdateTaskStatusRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let res = state
        .task_service
        .update_task_status(&id, payload, &auth.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated task status", res);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/plan/tasks/{id}",
    tag = "Plan / Tasks",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Tugas berhasil dihapus", body = ApiResponse<()>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>),
        (status = 403, description = "Forbidden", body = ApiResponse<()>),
        (status = 404, description = "Not Found", body = ApiResponse<()>)
    )
)]
pub async fn delete_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.task_service.delete_task(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted farm task", ());
    Ok((status, body))
}
