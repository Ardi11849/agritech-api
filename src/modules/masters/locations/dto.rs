use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationEntity {
    pub id: String,
    pub name: String,
    pub manager_name: String,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSummaryRow {
    pub id: String,
    pub name: String,
    pub manager_name: String,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
    pub installation_count: i64,
    pub active_installation_count: i64,
    pub total_capacity_holes: i64,
    pub active_capacity_holes: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateLocationRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Greenhouse Lembang 1")]
    pub name: String,

    #[validate(length(max = 100, message = "Manager name cannot exceed 100 characters"))]
    #[schema(example = "Budi Santoso")]
    pub manager_name: Option<String>,

    #[validate(length(max = 500, message = "Address cannot exceed 500 characters"))]
    #[schema(example = "Jl. Maribaya No. 12, Lembang, Bandung Barat")]
    pub address: Option<String>,

    pub is_active: Option<bool>,
}

pub type UpdateLocationRequest = CreateLocationRequest;

#[derive(Debug, Deserialize, IntoParams)]
pub struct LocationListQuery {
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationResponse {
    pub id: String,
    pub name: String,
    pub manager_name: String,
    pub address: String,
    pub is_active: bool,
    pub installation_count: i64,
    pub active_installation_count: i64,
    pub total_capacity_holes: i64,
    pub active_capacity_holes: i64,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}
