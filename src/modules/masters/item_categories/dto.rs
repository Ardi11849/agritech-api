use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemCategoryEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemCategorySummaryRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
    pub item_count: i64,
    pub active_item_count: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateItemCategoryRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Nutrisi & Pupuk")]
    pub name: String,

    #[validate(length(max = 500, message = "Description cannot exceed 500 characters"))]
    #[schema(example = "Larutan nutrisi AB Mix, pH Up/Down, dan pupuk pelengkap")]
    pub description: Option<String>,

    pub is_active: Option<bool>,
}

pub type UpdateItemCategoryRequest = CreateItemCategoryRequest;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ItemCategoryListQuery {
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ItemCategoryResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub item_count: i64,
    pub active_item_count: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}
