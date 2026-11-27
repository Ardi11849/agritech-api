use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AllocationType {
    Direct,
    Shared,
    Asset,
}

impl fmt::Display for AllocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationType::Direct => write!(f, "Direct"),
            AllocationType::Shared => write!(f, "Shared"),
            AllocationType::Asset => write!(f, "Asset"),
        }
    }
}

impl FromStr for AllocationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "direct" => Ok(AllocationType::Direct),
            "shared" => Ok(AllocationType::Shared),
            "asset" => Ok(AllocationType::Asset),
            _ => Err(format!("Unknown allocation type: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PurchaseItemCalcInput {
    pub quantity: Decimal,
    pub price_per_unit: Decimal,
    pub allocation_type: AllocationType,
    pub allocation_days: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct PurchaseItemDistribution {
    pub line_number: i32,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub shipping_cost: Decimal,
    pub total_cost: Decimal,
    pub effective_unit_cost: Decimal,
    pub allocation_days: i32,
    pub daily_allocation_rate: Decimal,
}

#[derive(Debug, Clone)]
pub struct PurchaseDistributionResult {
    pub subtotal_amount: Decimal,
    pub global_discount: Decimal,
    pub global_shipping: Decimal,
    pub total_amount: Decimal,
    pub items: Vec<PurchaseItemDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePurchaseItemRequest {
    pub item_id: Option<String>,
    pub item_name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub quantity: Decimal,
    pub price_per_unit: Decimal,
    pub allocation_type: AllocationType,
    pub session_id: Option<String>,
    pub allocation_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePurchaseRequest {
    pub transaction_date: Option<NaiveDate>,
    pub supplier_id: Option<String>,
    pub supplier_name: Option<String>,
    pub location_id: String,
    pub global_discount: Option<Decimal>,
    pub global_shipping: Option<Decimal>,
    pub notes: Option<String>,
    pub items: Vec<CreatePurchaseItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PurchaseItemResponse {
    pub id: String,
    pub purchase_id: String,
    pub line_number: i32,
    pub item_id: Option<String>,
    pub item_name: String,
    pub category: String,
    pub unit: String,
    pub quantity: Decimal,
    pub price_per_unit: Decimal,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub shipping_cost: Decimal,
    pub total_cost: Decimal,
    pub effective_unit_cost: Decimal,
    pub allocation_type: AllocationType,
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub allocation_days: i32,
    pub daily_allocation_rate: Decimal,
    pub is_synced_to_inventory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PurchaseResponse {
    pub id: String,
    pub invoice_number: String,
    pub transaction_date: NaiveDate,
    pub supplier_id: Option<String>,
    pub supplier_name: String,
    pub location_id: String,
    pub location_name: String,
    pub subtotal_amount: Decimal,
    pub global_discount: Decimal,
    pub global_shipping: Decimal,
    pub total_amount: Decimal,
    pub item_count: i32,
    pub notes: String,
    pub items: Vec<PurchaseItemResponse>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PurchaseKpi {
    pub total_purchases_count: i64,
    pub total_spend_amount: Decimal,
    pub direct_cost_total: Decimal,
    pub shared_and_asset_cost_total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PurchaseListResponse {
    pub kpi: PurchaseKpi,
    pub items: Vec<PurchaseResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct PurchaseQuery {
    pub location_id: Option<String>,
    pub supplier_id: Option<String>,
    pub allocation_type: Option<AllocationType>,
    pub session_id: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
}
