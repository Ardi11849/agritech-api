use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum CashflowEntryType {
    INFLOW,
    OUTFLOW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum CashflowTypeFilter {
    ALL,
    INFLOW,
    OUTFLOW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum SortOrder {
    ASC,
    DESC,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct CashflowSummaryQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub location_id: Option<String>,
    pub type_filter: Option<CashflowTypeFilter>,
    pub realized_only: Option<bool>,
    pub search: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CashflowPeriodMeta {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub location_id: Option<String>,
    pub realized_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CashflowSummaryKpi {
    pub total_gross_sales: Decimal,
    pub total_shipping_subsidy: Decimal,
    pub total_inflow: Decimal,
    pub total_outflow: Decimal,
    pub net_cashflow: Decimal,
    pub profit_margin_percent: Decimal,
    pub inflow_count: i64,
    pub outflow_count: i64,
    pub total_entries: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CashflowLedgerEntry {
    pub id: String,
    pub date: NaiveDate,
    pub entry_type: CashflowEntryType,
    pub category: String,
    pub reference_number: String,
    pub counterparty: String,
    pub location_id: String,
    pub location_name: String,
    pub description: String,
    pub gross_amount: Decimal,
    pub shipping_subsidy: Decimal,
    pub amount: Decimal,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub target_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CashflowSummaryResponse {
    pub period: CashflowPeriodMeta,
    pub kpi: CashflowSummaryKpi,
    pub entries: Vec<CashflowLedgerEntry>,
}
