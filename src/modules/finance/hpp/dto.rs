use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct HppListQuery {
    pub location_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub date: Option<NaiveDate>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HppSummaryKpi {
    pub total_cumulative_hpp: Decimal,
    pub total_cumulative_revenue: Decimal,
    pub total_cumulative_shipping_subsidy: Decimal,
    pub total_cumulative_net_profit: Decimal,
    pub average_profit_margin_percent: Decimal,
    pub profitable_sessions_count: i64,
    pub loss_sessions_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HppSessionItem {
    pub session_id: String,
    pub session_name: String,
    pub plant_name: String,
    pub plant_variety: String,
    pub location_id: String,
    pub location_name: String,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub harvest_date: Option<NaiveDate>,
    pub duration_days: i64,
    pub total_holes: i32,
    pub waste_qty: i32,
    pub waste_percentage: Decimal,
    pub direct_costs: Decimal,
    pub allocated_shared_costs: Decimal,
    pub allocated_asset_costs: Decimal,
    pub allocated_costs: Decimal,
    pub total_hpp: Decimal,
    pub total_revenue: Decimal,
    pub total_shipping_subsidy: Decimal,
    pub net_revenue: Decimal,
    pub net_profit: Decimal,
    pub profit_margin_percent: Decimal,
    pub effective_yield_kg: Decimal,
    pub cost_per_kg: Decimal,
    pub net_revenue_per_kg: Decimal,
    pub profit_per_kg: Decimal,
    pub is_snapshot_locked: bool,
    pub hpp_snapshot_locked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HppListResponse {
    pub kpi: HppSummaryKpi,
    pub items: Vec<HppSessionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HppSnapshotInfo {
    pub final_hpp: Option<Decimal>,
    pub final_revenue: Option<Decimal>,
    pub final_shipping_subsidy: Option<Decimal>,
    pub final_net_profit: Option<Decimal>,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RHPPDirectPurchaseItem {
    pub purchase_item_id: String,
    pub purchase_id: String,
    pub invoice_number: String,
    pub transaction_date: NaiveDate,
    pub item_name: String,
    pub category: String,
    pub unit: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
    pub allocation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RHPPAllocatedPurchaseItem {
    pub purchase_item_id: String,
    pub purchase_id: String,
    pub invoice_number: String,
    pub transaction_date: NaiveDate,
    pub item_name: String,
    pub category: String,
    pub allocation_type: String,
    pub item_total_cost: Decimal,
    pub allocation_days: i32,
    pub daily_rate: Decimal,
    pub overlap_days: i64,
    pub overlap_start_date: NaiveDate,
    pub overlap_end_date: NaiveDate,
    pub average_ratio_percent: Decimal,
    pub allocated_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RHPPRealizedSaleItem {
    pub sale_id: String,
    pub sale_number: String,
    pub sale_date: NaiveDate,
    pub customer_name: String,
    pub unit: String,
    pub quantity: Decimal,
    pub price_per_unit: Decimal,
    pub gross_revenue: Decimal,
    pub discount: Decimal,
    pub total_revenue: Decimal,
    pub seller_shipping_subsidy: Decimal,
    pub net_proceeds: Decimal,
    pub payment_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RHPPDetailResponse {
    pub summary: HppSessionItem,
    pub snapshot: HppSnapshotInfo,
    pub direct_purchase_items: Vec<RHPPDirectPurchaseItem>,
    pub allocated_purchase_items: Vec<RHPPAllocatedPurchaseItem>,
    pub realized_sales: Vec<RHPPRealizedSaleItem>,
}

#[derive(Debug, Clone)]
pub struct HppSessionCalcInput {
    pub session_id: String,
    pub session_name: String,
    pub plant_name: String,
    pub plant_variety: String,
    pub location_id: String,
    pub location_name: String,
    pub status: String,
    pub start_date: NaiveDate,
    pub harvest_date: Option<NaiveDate>,
    pub total_holes: i32,
    pub mortality_count: i32,
    pub actual_yield_kg: Decimal,
    pub final_hpp: Option<Decimal>,
    pub final_revenue: Option<Decimal>,
    pub final_shipping_subsidy: Option<Decimal>,
    pub final_net_profit: Option<Decimal>,
    pub hpp_snapshot_locked_at: Option<DateTime<Utc>>,
    pub hpp_snapshot_locked_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HppPurchaseCalcInput {
    pub purchase_item_id: String,
    pub purchase_id: String,
    pub invoice_number: String,
    pub transaction_date: NaiveDate,
    pub location_id: String,
    pub item_name: String,
    pub category: String,
    pub unit: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
    pub allocation_type: String,
    pub allocation_days: i32,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HppSaleCalcInput {
    pub sale_id: String,
    pub sale_number: String,
    pub sale_date: NaiveDate,
    pub session_id: String,
    pub customer_name: String,
    pub unit: String,
    pub quantity: Decimal,
    pub price_per_unit: Decimal,
    pub gross_revenue: Decimal,
    pub discount: Decimal,
    pub total_revenue: Decimal,
    pub seller_shipping_subsidy: Decimal,
    pub net_proceeds: Decimal,
    pub payment_status: String,
}
