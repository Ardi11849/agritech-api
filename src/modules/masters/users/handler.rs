use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::common::extractor::AppJson;
use crate::common::response::ApiResponse;
use crate::middleware::auth::AuthUser;
use crate::modules::user::dto::{CreateUserRequest, UpdateUserRequest};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/users/profile",
    tag = "User Profile",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = ApiResponse<UserProfileResponse>),
        (status = 401, description = "Unauthorized - Missing or invalid token", body = ApiResponse<()>)
    )
)]
pub async fn get_profile(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let profile = state.user_service.get_profile(&auth_user.user_id).await?;
    let (status, body) = ApiResponse::success(
        StatusCode::OK,
        "Successfully retrieved user profile",
        profile,
    );
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "User Management",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved users", body = ApiResponse<Vec<UserItemResponse>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<()>)
    )
)]
pub async fn get_users(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let users = state.user_service.get_users().await?;
    let (status, body) =
        ApiResponse::success(StatusCode::OK, "Successfully retrieved users", users);
    Ok((status, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "User Management",
    params(("id" = String, Path, description = "User ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully retrieved user detail", body = ApiResponse<UserItemResponse>),
        (status = 404, description = "User not found", body = ApiResponse<()>)
    )
)]
pub async fn get_user(
    _auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let user = state.user_service.get_user_by_id(&id).await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully retrieved user", user);
    Ok((status, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "User Management",
    request_body = CreateUserRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 201, description = "Successfully created user", body = ApiResponse<UserItemResponse>),
        (status = 400, description = "Validation error or role not found", body = ApiResponse<()>),
        (status = 409, description = "Username or email already exists", body = ApiResponse<()>)
    )
)]
pub async fn create_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    AppJson(payload): AppJson<CreateUserRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let user = state
        .user_service
        .create_user(payload, &auth_user.username)
        .await?;
    let (status, body) =
        ApiResponse::success(StatusCode::CREATED, "Successfully created user", user);
    Ok((status, body))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "User Management",
    params(("id" = String, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully updated user", body = ApiResponse<UserItemResponse>),
        (status = 400, description = "Validation error", body = ApiResponse<()>),
        (status = 404, description = "User not found", body = ApiResponse<()>),
        (status = 409, description = "Username or email conflict", body = ApiResponse<()>)
    )
)]
pub async fn update_user(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    AppJson(payload): AppJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    let user = state
        .user_service
        .update_user(&id, payload, &auth_user.username)
        .await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully updated user", user);
    Ok((status, body))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "User Management",
    params(("id" = String, Path, description = "User ID")),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Successfully deleted user", body = ApiResponse<()>),
        (status = 400, description = "Self-deletion or superadmin deletion rejected", body = ApiResponse<()>),
        (status = 404, description = "User not found", body = ApiResponse<()>)
    )
)]
pub async fn delete_user(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::common::error::AppError> {
    state
        .user_service
        .delete_user(&id, &auth_user.user_id)
        .await?;
    let (status, body) = ApiResponse::success(StatusCode::OK, "Successfully deleted user", ());
    Ok((status, body))
}
