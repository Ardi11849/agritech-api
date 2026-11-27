use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockHealthStatus {
    OutOfStock,
    LowStock,
    Safe,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MasterItemEntity {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub default_unit: String,
    pub min_stock_level: Decimal,
    pub current_stock: Decimal,
    pub average_unit_cost: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MasterItemRow {
    pub id: String,
    pub category_id: String,
    pub category_name: String,
    pub name: String,
    pub default_unit: String,
    pub min_stock_level: Decimal,
    pub current_stock: Decimal,
    pub average_unit_cost: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateMasterItemRequest {
    #[validate(length(min = 1, max = 50, message = "ID kategori wajib diisi"))]
    pub category_id: String,
    #[validate(length(min = 1, max = 150, message = "Nama barang harus 1-150 karakter"))]
    pub name: String,
    #[validate(length(max = 30, message = "Satuan dasar maksimal 30 karakter"))]
    pub default_unit: Option<String>,
    #[schema(value_type = f64, example = 5.25)]
    pub min_stock_level: Option<Decimal>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMasterItemRequest {
    #[validate(length(min = 1, max = 50, message = "ID kategori wajib diisi"))]
    pub category_id: String,
    #[validate(length(min = 1, max = 150, message = "Nama barang harus 1-150 karakter"))]
    pub name: String,
    #[validate(length(max = 30, message = "Satuan dasar maksimal 30 karakter"))]
    pub default_unit: Option<String>,
    #[schema(value_type = f64, example = 5.25)]
    pub min_stock_level: Option<Decimal>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct ItemListQuery {
    pub category_id: Option<String>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MasterItemResponse {
    pub id: String,
    pub category_id: String,
    pub category_name: String,
    pub name: String,
    pub default_unit: String,
    #[schema(value_type = f64, example = 5.26)]
    pub min_stock_level: Decimal,
    #[schema(value_type = f64, example = 7.50)]
    pub current_stock: Decimal,
    #[schema(value_type = f64, example = 27000.00)]
    pub average_unit_cost: Decimal,
    pub stock_health: StockHealthStatus,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

impl From<MasterItemRow> for MasterItemResponse {
    fn from(row: MasterItemRow) -> Self {
        let stock_health = crate::modules::masters::items::service::evaluate_stock_health(
            row.current_stock,
            row.min_stock_level,
        );
        Self {
            id: row.id,
            category_id: row.category_id,
            category_name: row.category_name,
            name: row.name,
            default_unit: row.default_unit,
            min_stock_level: row.min_stock_level,
            current_stock: row.current_stock,
            average_unit_cost: row.average_unit_cost,
            stock_health,
            is_active: row.is_active,
            created_at: row.created_at,
            created_by: row.created_by,
            modify_at: row.modify_at,
            modify_by: row.modify_by,
        }
    }
}
