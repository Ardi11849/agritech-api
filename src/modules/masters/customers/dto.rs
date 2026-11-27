use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CustomerEntity {
    pub id: String,
    pub name: String,
    pub category: String,
    pub contact_person: String,
    pub phone: Option<String>,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCustomerRequest {
    #[validate(length(min = 1, max = 150, message = "Nama pelanggan harus 1-150 karakter"))]
    pub name: String,
    #[validate(length(max = 100, message = "Segmen pelanggan maksimal 100 karakter"))]
    pub category: Option<String>,
    #[validate(length(max = 100, message = "Nama kontak person maksimal 100 karakter"))]
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    #[validate(length(max = 500, message = "Alamat maksimal 500 karakter"))]
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCustomerRequest {
    #[validate(length(min = 1, max = 150, message = "Nama pelanggan harus 1-150 karakter"))]
    pub name: String,
    #[validate(length(max = 100, message = "Segmen pelanggan maksimal 100 karakter"))]
    pub category: Option<String>,
    #[validate(length(max = 100, message = "Nama kontak person maksimal 100 karakter"))]
    pub contact_person: Option<String>,
    pub phone: Option<String>,
    #[validate(length(max = 500, message = "Alamat maksimal 500 karakter"))]
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct CustomerListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomerResponse {
    pub id: String,
    pub name: String,
    pub category: String,
    pub contact_person: String,
    pub phone: Option<String>,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

impl From<CustomerEntity> for CustomerResponse {
    fn from(entity: CustomerEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            category: entity.category,
            contact_person: entity.contact_person,
            phone: entity.phone,
            address: entity.address,
            is_active: entity.is_active,
            created_at: entity.created_at,
            created_by: entity.created_by,
            modify_at: entity.modify_at,
            modify_by: entity.modify_by,
        }
    }
}
