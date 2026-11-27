use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateGroupRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateModuleRequest {
    #[validate(length(min = 1, message = "group_id is required"))]
    pub group_id: String,
    pub parent_id: Option<String>,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Path must be between 1 and 100 characters"
    ))]
    pub path: String,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateModuleRequest {
    pub group_id: Option<String>,
    pub parent_id: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct GroupEntity {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct ModuleEntity {
    pub id: String,
    pub group_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct ModuleTreeResponse {
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
    pub sub_modules: Vec<ModuleTreeResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub modify_at: DateTime<Utc>,
    pub modules: Vec<ModuleTreeResponse>,
}
