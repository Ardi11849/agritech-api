use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::role::dto::{
    CreateRolePermissionRequest, CreateRoleRequest, UpdateRolePermissionRequest, UpdateRoleRequest,
};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/roles",
    tag = "Access Control / RBAC",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved roles", body = ApiResponse<Vec<RoleItemResponse>>)
    )
)]
pub async fn get_roles(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let roles = state.role_service.get_roles().await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved roles", roles);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/roles/{id}",
    tag = "Access Control / RBAC",
    params(("id" = String, Path, description = "Role ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved role detail with modules", body = ApiResponse<RoleDetailResponse>),
        (status = 404, description = "Role not found", body = ApiResponse<()>)
    )
)]
pub async fn get_role(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let role = state.role_service.get_role_detail(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully retrieved role", role);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/roles",
    tag = "Access Control / RBAC",
    request_body = CreateRoleRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 201, description = "Successfully created role", body = ApiResponse<RoleItemResponse>),
        (status = 400, description = "Validation error", body = ApiResponse<()>)
    )
)]
pub async fn create_role(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    AppJson(payload): AppJson<CreateRoleRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let role = state.role_service.create_role(payload).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created role", role);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/roles/{id}",
    tag = "Access Control / RBAC",
    params(("id" = String, Path, description = "Role ID")),
    request_body = UpdateRoleRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully updated role", body = ApiResponse<RoleItemResponse>),
        (status = 404, description = "Role not found", body = ApiResponse<()>)
    )
)]
pub async fn update_role(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    AppJson(payload): AppJson<UpdateRoleRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let role = state.role_service.update_role(&id, payload).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully updated role", role);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/roles/{id}",
    tag = "Access Control / RBAC",
    params(("id" = String, Path, description = "Role ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully deleted role", body = ApiResponse<()>)
    )
)]
pub async fn delete_role(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.role_service.delete_role(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted role", ());
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/roles/rolePermissions",
    tag = "Access Control / RBAC",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved permissions", body = ApiResponse<Vec<RolePermissionDto>>)
    )
)]
pub async fn get_all_permissions(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let perms = state.role_permission_service.get_all_permissions().await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved permissions", perms);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/roles/rolePermissions/{id}",
    tag = "Access Control / RBAC",
    params(("id" = String, Path, description = "Permission ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved permission", body = ApiResponse<RolePermissionDto>),
        (status = 404, description = "Permission not found", body = ApiResponse<()>)
    )
)]
pub async fn get_permission_by_id(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let perm = state
        .role_permission_service
        .get_permission_by_id(&id)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved permission", perm);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/roles/rolePermissions/role/{id_role}",
    tag = "Access Control / RBAC",
    params(("id_role" = String, Path, description = "Role ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved role permissions", body = ApiResponse<Vec<RolePermissionDto>>)
    )
)]
pub async fn get_permissions_by_role(
    _auth_user: AuthUser,
    Path(id_role): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let perms = state
        .role_permission_service
        .get_permissions_by_role(&id_role)
        .await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved role permissions",
        perms,
    );
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/roles/rolePermissions",
    tag = "Access Control / RBAC",
    request_body = CreateRolePermissionRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 201, description = "Successfully created permission", body = ApiResponse<RolePermissionDto>),
        (status = 400, description = "Validation or duplicate error", body = ApiResponse<()>)
    )
)]
pub async fn create_permission(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    AppJson(payload): AppJson<CreateRolePermissionRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let perm = state
        .role_permission_service
        .create_permission(payload)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created permission", perm);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/roles/rolePermissions/{id}",
    tag = "Access Control / RBAC",
    params(("id" = String, Path, description = "Permission ID")),
    request_body = UpdateRolePermissionRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully updated permission", body = ApiResponse<RolePermissionDto>),
        (status = 404, description = "Permission not found", body = ApiResponse<()>)
    )
)]
pub async fn update_permission(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    AppJson(payload): AppJson<UpdateRolePermissionRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let perm = state
        .role_permission_service
        .update_permission(&id, payload)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully updated permission", perm);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/roles/rolePermissions/{id}",
    tag = "Access Control / RBAC",
    params(("id" = String, Path, description = "Permission ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully deleted permission", body = ApiResponse<()>)
    )
)]
pub async fn delete_permission(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state.role_permission_service.delete_permission(&id).await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully deleted permission", ());
    Ok((status, body))
}
