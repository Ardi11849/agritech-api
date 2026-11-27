use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRoleRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Role name must be between 1 and 50 characters"
    ))]
    pub name: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Role name must be between 1 and 50 characters"
    ))]
    pub name: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct RoleEntity {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct RoleItemResponse {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub modify_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct RolePermissionEntity {
    pub id: String,
    pub role_id: String,
    pub module_group_id: String,
    pub module_id: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct RolePermissionDto {
    pub id: String,
    pub role_id: String,
    pub module_group_id: String,
    pub module_id: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
    pub modify_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct ModuleDetailDto {
    pub id: String,
    pub group_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub path: String,
    pub icon: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
    pub sub_modules: Vec<ModuleDetailDto>,
    pub role_permission: Vec<RolePermissionDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct ModuleGroupDetailDto {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub modify_at: DateTime<Utc>,
    pub module: Vec<ModuleDetailDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct RoleDetailResponse {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub modify_at: DateTime<Utc>,
    pub module_group: Vec<ModuleGroupDetailDto>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateRolePermissionRequest {
    #[validate(length(min = 1, message = "role_id is required"))]
    pub role_id: String,
    #[validate(length(min = 1, message = "module_id is required"))]
    pub module_id: String,
    pub module_group_id: Option<String>,
    pub can_create: Option<bool>,
    pub can_read: Option<bool>,
    pub can_update: Option<bool>,
    pub can_delete: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateRolePermissionRequest {
    pub can_create: Option<bool>,
    pub can_read: Option<bool>,
    pub can_update: Option<bool>,
    pub can_delete: Option<bool>,
}
