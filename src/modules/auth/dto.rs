use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(email(message = "Invalid email format"), length(max = 100))]
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
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct RegisterResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub role_id: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct LoginUserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub role_id: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: LoginUserResponse,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserEntity {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub role_id: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}
