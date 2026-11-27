use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum TaxStatus {
    Unpaid,
    Paid,
}

impl fmt::Display for TaxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaxStatus::Unpaid => write!(f, "Unpaid"),
            TaxStatus::Paid => write!(f, "Paid"),
        }
    }
}

impl FromStr for TaxStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "paid" => Ok(TaxStatus::Paid),
            "unpaid" => Ok(TaxStatus::Unpaid),
            _ => Err(format!("Unknown tax status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum TaxAlertStatus {
    #[serde(rename = "PAID")]
    Paid,
    #[serde(rename = "NORMAL")]
    Normal,
    #[serde(rename = "WARNING_H7")]
    WarningH7,
    #[serde(rename = "OVERDUE")]
    Overdue,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaxSettingsDto {
    pub id: String,
    pub scheme_name: String,
    pub rate: Decimal,
    pub rate_percent: Decimal,
    pub description: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaxCurrentPeriodDto {
    pub report_id: Option<String>,
    pub period_month: i32,
    pub period_year: i32,
    pub total_revenue: Decimal,
    pub realized_sales_count: i64,
    pub tax_rate: Decimal,
    pub tax_rate_percent: Decimal,
    pub estimated_tax_amount: Decimal,
    pub status: TaxStatus,
    pub deadline_date: NaiveDate,
    pub days_remaining: i64,
    pub alert_status: TaxAlertStatus,
    pub payment_date: Option<DateTime<Utc>>,
    pub billing_code: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaxReportItem {
    pub id: String,
    pub period_month: i32,
    pub period_year: i32,
    pub total_revenue: Decimal,
    pub tax_rate: Decimal,
    pub tax_rate_percent: Decimal,
    pub tax_amount: Decimal,
    pub status: TaxStatus,
    pub payment_date: Option<DateTime<Utc>>,
    pub billing_code: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaxDashboardData {
    pub settings: TaxSettingsDto,
    pub current_period: TaxCurrentPeriodDto,
    pub tax_history: Vec<TaxReportItem>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct TaxDashboardQuery {
    pub month: Option<i32>,
    pub year: Option<i32>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PayTaxRequest {
    pub period_month: i32,
    pub period_year: i32,
    pub payment_date: Option<DateTime<Utc>>,
    pub billing_code: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTaxSettingsRequest {
    pub scheme_name: String,
    pub rate: Decimal,
    pub description: Option<String>,
}
