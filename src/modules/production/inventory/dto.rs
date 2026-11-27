use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::modules::masters::items::dto::StockHealthStatus;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MovementType {
    StockIn,
    StockOut,
    ReturnIn,
    ReturnOut,
    OpnameAdjustment,
}

impl MovementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MovementType::StockIn => "STOCK_IN",
            MovementType::StockOut => "STOCK_OUT",
            MovementType::ReturnIn => "RETURN_IN",
            MovementType::ReturnOut => "RETURN_OUT",
            MovementType::OpnameAdjustment => "OPNAME_ADJUSTMENT",
        }
    }
}

impl std::str::FromStr for MovementType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "STOCK_IN" => Ok(MovementType::StockIn),
            "STOCK_OUT" => Ok(MovementType::StockOut),
            "RETURN_IN" => Ok(MovementType::ReturnIn),
            "RETURN_OUT" => Ok(MovementType::ReturnOut),
            "OPNAME_ADJUSTMENT" => Ok(MovementType::OpnameAdjustment),
            _ => Err(format!("Unknown movement type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockMovementInputType {
    StockIn,
    StockOut,
    ReturnIn,
    ReturnOut,
}

impl From<StockMovementInputType> for MovementType {
    fn from(input: StockMovementInputType) -> Self {
        match input {
            StockMovementInputType::StockIn => MovementType::StockIn,
            StockMovementInputType::StockOut => MovementType::StockOut,
            StockMovementInputType::ReturnIn => MovementType::ReturnIn,
            StockMovementInputType::ReturnOut => MovementType::ReturnOut,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementCalculation {
    pub movement_type: MovementType,
    pub quantity: Decimal,
    pub delta_quantity: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub unit_cost: Decimal,
    pub average_unit_cost_after: Decimal,
    pub total_cost_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventoryKpi {
    pub total_sku: i64,
    pub safe_count: i64,
    pub low_stock_count: i64,
    pub out_of_stock_count: i64,
    #[schema(value_type = f64, example = 4850000.0)]
    pub total_valuation_rp: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventoryItemSummary {
    pub id: String,
    pub category_id: String,
    pub category_name: String,
    pub name: String,
    pub default_unit: String,
    #[schema(value_type = f64, example = 10.0)]
    pub min_stock_level: Decimal,
    #[schema(value_type = f64, example = 7.5)]
    pub current_stock: Decimal,
    #[schema(value_type = f64, example = 27000.0)]
    pub average_unit_cost: Decimal,
    #[schema(value_type = f64, example = 202500.0)]
    pub total_valuation_rp: Decimal,
    pub stock_health: StockHealthStatus,
    pub is_active: bool,
    pub modify_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventorySummaryResponse {
    pub kpi: InventoryKpi,
    pub items: Vec<InventoryItemSummary>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct InventorySummaryQuery {
    pub category_id: Option<String>,
    pub stock_health: Option<StockHealthStatus>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateMovementRequest {
    #[validate(length(min = 1, max = 50, message = "ID barang wajib diisi"))]
    pub item_id: String,
    pub movement_type: StockMovementInputType,
    #[schema(value_type = f64, example = 20.0)]
    pub quantity: Decimal,
    #[schema(value_type = f64, example = 25000.0)]
    pub unit_cost: Option<Decimal>,
    pub supplier_id: Option<String>,
    pub customer_id: Option<String>,
    pub installation_id: Option<String>,
    #[validate(length(max = 100, message = "Nomor referensi maksimal 100 karakter"))]
    pub reference_no: Option<String>,
    #[validate(length(max = 500, message = "Catatan maksimal 500 karakter"))]
    pub notes: Option<String>,
    pub transaction_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateStockOpnameRequest {
    #[validate(length(min = 1, max = 50, message = "ID barang wajib diisi"))]
    pub item_id: String,
    #[schema(value_type = f64, example = 6.75)]
    pub actual_physical_stock: Decimal,
    #[validate(length(max = 100, message = "Nomor referensi maksimal 100 karakter"))]
    pub reference_no: Option<String>,
    #[validate(length(
        min = 3,
        max = 500,
        message = "Catatan opname minimal 3 dan maksimal 500 karakter"
    ))]
    pub notes: String,
    pub transaction_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryLedgerRow {
    pub id: String,
    pub item_id: String,
    pub item_name: String,
    pub default_unit: String,
    pub movement_type: String,
    pub quantity: Decimal,
    pub delta_quantity: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub unit_cost: Decimal,
    pub average_unit_cost_after: Decimal,
    pub total_cost_value: Decimal,
    pub min_stock_level: Decimal,
    pub supplier_id: Option<String>,
    pub supplier_name: Option<String>,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub installation_id: Option<String>,
    pub installation_name: Option<String>,
    pub reference_no: String,
    pub notes: String,
    pub transaction_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InventoryLedgerResponse {
    pub id: String,
    pub item_id: String,
    pub item_name: String,
    pub default_unit: String,
    pub movement_type: MovementType,
    #[schema(value_type = f64, example = 22.5)]
    pub quantity: Decimal,
    #[schema(value_type = f64, example = -22.5)]
    pub delta_quantity: Decimal,
    #[schema(value_type = f64, example = 30.0)]
    pub balance_before: Decimal,
    #[schema(value_type = f64, example = 7.5)]
    pub balance_after: Decimal,
    #[schema(value_type = f64, example = 27000.0)]
    pub unit_cost: Decimal,
    #[schema(value_type = f64, example = 27000.0)]
    pub average_unit_cost_after: Decimal,
    #[schema(value_type = f64, example = 607500.0)]
    pub total_cost_value: Decimal,
    pub stock_health: StockHealthStatus,
    pub supplier_id: Option<String>,
    pub supplier_name: Option<String>,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub installation_id: Option<String>,
    pub installation_name: Option<String>,
    pub reference_no: String,
    pub notes: String,
    pub transaction_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl From<InventoryLedgerRow> for InventoryLedgerResponse {
    fn from(row: InventoryLedgerRow) -> Self {
        let movement_type = row
            .movement_type
            .parse::<MovementType>()
            .unwrap_or(MovementType::StockIn);
        let stock_health = crate::modules::masters::items::service::evaluate_stock_health(
            row.balance_after,
            row.min_stock_level,
        );

        Self {
            id: row.id,
            item_id: row.item_id,
            item_name: row.item_name,
            default_unit: row.default_unit,
            movement_type,
            quantity: row.quantity,
            delta_quantity: row.delta_quantity,
            balance_before: row.balance_before,
            balance_after: row.balance_after,
            unit_cost: row.unit_cost,
            average_unit_cost_after: row.average_unit_cost_after,
            total_cost_value: row.total_cost_value,
            stock_health,
            supplier_id: row.supplier_id,
            supplier_name: row.supplier_name,
            customer_id: row.customer_id,
            customer_name: row.customer_name,
            installation_id: row.installation_id,
            installation_name: row.installation_name,
            reference_no: row.reference_no,
            notes: row.notes,
            transaction_date: row.transaction_date,
            created_at: row.created_at,
            created_by: row.created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct StockCardQuery {
    pub movement_type: Option<MovementType>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StockCardResponse {
    pub item: InventoryItemSummary,
    pub total_movements: i64,
    pub movements: Vec<InventoryLedgerResponse>,
}
