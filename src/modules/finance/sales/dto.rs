use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum SaleStatus {
    Order,
    Realized,
}

impl fmt::Display for SaleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaleStatus::Order => write!(f, "Order"),
            SaleStatus::Realized => write!(f, "Realized"),
        }
    }
}

impl FromStr for SaleStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "order" => Ok(SaleStatus::Order),
            "realized" => Ok(SaleStatus::Realized),
            _ => Err(format!("Unknown sale status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum PaymentStatus {
    Lunas,
    Piutang,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Lunas => write!(f, "Lunas"),
            PaymentStatus::Piutang => write!(f, "Piutang"),
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lunas" => Ok(PaymentStatus::Lunas),
            "piutang" => Ok(PaymentStatus::Piutang),
            _ => Err(format!("Unknown payment status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ShippingBorneBy {
    Pembeli,
    Penjual,
}

impl fmt::Display for ShippingBorneBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShippingBorneBy::Pembeli => write!(f, "Pembeli"),
            ShippingBorneBy::Penjual => write!(f, "Penjual"),
        }
    }
}

impl FromStr for ShippingBorneBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "pembeli" => Ok(ShippingBorneBy::Pembeli),
            "penjual" => Ok(ShippingBorneBy::Penjual),
            _ => Err(format!("Unknown shipping borne by: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaleCalcInput {
    pub quantity: Decimal,
    pub requested_quantity: Option<Decimal>,
    pub price_per_unit: Decimal,
    pub discount: Option<Decimal>,
    pub shipping_cost: Option<Decimal>,
    pub shipping_borne_by: ShippingBorneBy,
    pub status: SaleStatus,
}

#[derive(Debug, Clone)]
pub struct SaleFinancialResult {
    pub gross_revenue: Decimal,
    pub discount: Decimal,
    pub total_revenue: Decimal,
    pub shipping_cost: Decimal,
    pub seller_shipping_subsidy: Decimal,
    pub net_proceeds: Decimal,
    pub fulfillment_rate_percent: Decimal,
    pub requested_quantity: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSaleRequest {
    pub sale_date: Option<NaiveDate>,
    pub session_id: String,
    pub location_id: Option<String>,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub sales_channel: Option<String>,
    pub unit: Option<String>,
    pub requested_quantity: Option<Decimal>,
    #[serde(default)]
    pub quantity: Option<Decimal>,
    pub price_per_unit: Decimal,
    pub discount: Option<Decimal>,
    pub shipping_cost: Option<Decimal>,
    pub shipping_borne_by: Option<ShippingBorneBy>,
    pub status: Option<SaleStatus>,
    pub payment_status: Option<PaymentStatus>,
    pub notes: Option<String>,
}

pub type UpdateSaleRequest = CreateSaleRequest;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaleResponse {
    pub id: String,
    pub sale_number: String,
    pub sale_date: NaiveDate,
    pub session_id: String,
    pub session_name: String,
    pub plant_name: String,
    pub location_id: String,
    pub location_name: String,
    pub customer_id: Option<String>,
    pub customer_name: String,
    pub sales_channel: String,
    pub unit: String,
    pub requested_quantity: Decimal,
    pub quantity: Decimal,
    pub fulfillment_rate_percent: Decimal,
    pub price_per_unit: Decimal,
    pub gross_revenue: Decimal,
    pub discount: Decimal,
    pub total_revenue: Decimal,
    pub shipping_cost: Decimal,
    pub shipping_borne_by: ShippingBorneBy,
    pub seller_shipping_subsidy: Decimal,
    pub net_proceeds: Decimal,
    pub status: SaleStatus,
    pub payment_status: PaymentStatus,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaleKpi {
    pub total_realized_revenue: Decimal,
    pub total_receivable_amount: Decimal,
    pub total_sold_kg: Decimal,
    pub total_shipping_subsidy: Decimal,
    pub order_pipeline_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaleListResponse {
    pub kpi: SaleKpi,
    pub items: Vec<SaleResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct SaleQuery {
    pub session_id: Option<String>,
    pub location_id: Option<String>,
    pub customer_id: Option<String>,
    pub status: Option<SaleStatus>,
    pub payment_status: Option<PaymentStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
}
