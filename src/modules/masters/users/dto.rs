use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct UserProfileResponse {
    pub user_id: String,
    pub username: String,
    pub role_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, PartialEq, Eq, ToSchema)]
pub struct UserItemResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub role_id: String,
    pub role_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(
        email(message = "Invalid email format"),
        length(max = 100, message = "Email max 100 characters")
    )]
    pub email: String,

    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,

    #[validate(length(min = 1, message = "Role ID is required"))]
    pub role_id: String,

    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(
        email(message = "Invalid email format"),
        length(max = 100, message = "Email max 100 characters")
    )]
    pub email: String,

    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,

    pub password: Option<String>,

    #[validate(length(min = 1, message = "Role ID is required"))]
    pub role_id: String,

    pub is_active: Option<bool>,
}
