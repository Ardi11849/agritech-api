use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::system_modules::dto::{
    CreateGroupRequest, CreateModuleRequest, UpdateGroupRequest, UpdateModuleRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/system-modules/groups",
    tag = "Navigation / System Modules",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved module groups", body = ApiResponse<Vec<GroupResponse>>)
    )
)]
pub async fn get_groups(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let groups = state.system_modules_service.get_module_groups().await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved module groups",
        groups,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/system-modules/groups/{id}",
    tag = "Navigation / System Modules",
    params(("id" = String, Path, description = "Group ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved module group", body = ApiResponse<GroupResponse>),
        (status = 404, description = "Group not found", body = ApiResponse<()>)
    )
)]
pub async fn get_group_by_id(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let group = state.system_modules_service.get_group_by_id(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved module group", group);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/system-modules/groups",
    tag = "Navigation / System Modules",
    request_body = CreateGroupRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 201, description = "Successfully created module group", body = ApiResponse<GroupResponse>),
        (status = 400, description = "Validation error", body = ApiResponse<()>)
    )
)]
pub async fn create_group(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    AppJson(payload): AppJson<CreateGroupRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let group = state.system_modules_service.create_group(payload).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::CREATED,
        "Successfully created module group",
        group,
    );
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/system-modules/groups/{id}",
    tag = "Navigation / System Modules",
    params(("id" = String, Path, description = "Group ID")),
    request_body = UpdateGroupRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully updated module group", body = ApiResponse<GroupResponse>),
        (status = 404, description = "Group not found", body = ApiResponse<()>)
    )
)]
pub async fn update_group(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    AppJson(payload): AppJson<UpdateGroupRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let group = state
        .system_modules_service
        .update_group(&id, payload)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated module group", group);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/system-modules/groups/{id}",
    tag = "Navigation / System Modules",
    params(("id" = String, Path, description = "Group ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully deleted module group", body = ApiResponse<()>)
    )
)]
pub async fn delete_group(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.system_modules_service.delete_group(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted module group", ());
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/system-modules/modules",
    tag = "Navigation / System Modules",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved all modules", body = ApiResponse<Vec<ModuleTreeResponse>>)
    )
)]
pub async fn get_modules(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let modules = state.system_modules_service.get_all_modules().await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved all modules",
        modules,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/system-modules/modules/{id}",
    tag = "Navigation / System Modules",
    params(("id" = String, Path, description = "Module ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved module", body = ApiResponse<ModuleTreeResponse>),
        (status = 404, description = "Module not found", body = ApiResponse<()>)
    )
)]
pub async fn get_module_by_id(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let module = state.system_modules_service.get_module_by_id(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved module", module);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/system-modules/modules",
    tag = "Navigation / System Modules",
    request_body = CreateModuleRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 201, description = "Successfully created module", body = ApiResponse<ModuleTreeResponse>),
        (status = 400, description = "Validation error or group not found", body = ApiResponse<()>)
    )
)]
pub async fn create_module(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    AppJson(payload): AppJson<CreateModuleRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let module = state.system_modules_service.create_module(payload).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created module", module);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/system-modules/modules/{id}",
    tag = "Navigation / System Modules",
    params(("id" = String, Path, description = "Module ID")),
    request_body = UpdateModuleRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully updated module", body = ApiResponse<ModuleTreeResponse>),
        (status = 400, description = "Circular dependency or group not found", body = ApiResponse<()>),
        (status = 404, description = "Module not found", body = ApiResponse<()>)
    )
)]
pub async fn update_module(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    AppJson(payload): AppJson<UpdateModuleRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let module = state
        .system_modules_service
        .update_module(&id, payload)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated module", module);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/system-modules/modules/{id}",
    tag = "Navigation / System Modules",
    params(("id" = String, Path, description = "Module ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully deleted module", body = ApiResponse<()>)
    )
)]
pub async fn delete_module(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.system_modules_service.delete_module(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted module", ());
    Ok((status, body))
}
