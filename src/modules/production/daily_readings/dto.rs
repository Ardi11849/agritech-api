use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::modules::sessions::dto::{GrowthInfo, SessionItemResponse, SessionStatus};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricStatus {
    Ideal,
    Warning,
    Critical,
}

impl fmt::Display for MetricStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ideal => write!(f, "IDEAL"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl FromStr for MetricStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IDEAL" | "Ideal" => Ok(Self::Ideal),
            "WARNING" | "Warning" => Ok(Self::Warning),
            "CRITICAL" | "Critical" => Ok(Self::Critical),
            other => Err(format!("Unknown metric status: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ActiveMetricTargets {
    #[schema(value_type = f64, example = 700.0)]
    pub ppm_ideal_min: Decimal,
    #[schema(value_type = f64, example = 840.0)]
    pub ppm_ideal_max: Decimal,
    #[schema(value_type = f64, example = 600.0)]
    pub ppm_tol_min: Decimal,
    #[schema(value_type = f64, example = 940.0)]
    pub ppm_tol_max: Decimal,
    #[schema(value_type = f64, example = 5.5)]
    pub ph_ideal_min: Decimal,
    #[schema(value_type = f64, example = 6.5)]
    pub ph_ideal_max: Decimal,
    #[schema(value_type = f64, example = 5.0)]
    pub ph_tol_min: Decimal,
    #[schema(value_type = f64, example = 7.0)]
    pub ph_tol_max: Decimal,
    #[schema(value_type = f64, example = 18.0)]
    pub temp_env_ideal_min: Decimal,
    #[schema(value_type = f64, example = 26.0)]
    pub temp_env_ideal_max: Decimal,
    #[schema(value_type = f64, example = 18.0)]
    pub temp_water_ideal_min: Decimal,
    #[schema(value_type = f64, example = 24.0)]
    pub temp_water_ideal_max: Decimal,
    #[schema(value_type = f64, example = 60.0)]
    pub hum_ideal_min: Decimal,
    #[schema(value_type = f64, example = 80.0)]
    pub hum_ideal_max: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DailyReadingDiagnostics {
    pub ppm_status: MetricStatus,
    pub ph_status: MetricStatus,
    pub temp_env_status: Option<MetricStatus>,
    pub temp_water_status: Option<MetricStatus>,
    pub humidity_status: Option<MetricStatus>,
    pub overall_status: MetricStatus,
    pub active_targets: ActiveMetricTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DailyReadingItem {
    pub id: String,
    pub session_id: String,
    #[schema(value_type = String, example = "2026-11-11")]
    pub reading_date: NaiveDate,
    #[schema(value_type = String, example = "08:30:00")]
    pub reading_time: String,
    #[schema(value_type = f64, example = 750.0)]
    pub ppm: Decimal,
    #[schema(value_type = f64, example = 6.2)]
    pub ph: Decimal,
    #[schema(value_type = Option<f64>, example = 24.5)]
    pub temp_env: Option<Decimal>,
    #[schema(value_type = Option<f64>, example = 21.0)]
    pub temp_water: Option<Decimal>,
    #[schema(value_type = Option<f64>, example = 68.0)]
    pub humidity: Option<Decimal>,
    pub mortality_count: i32,
    pub notes: String,
    pub growth_info: GrowthInfo,
    pub diagnostics: DailyReadingDiagnostics,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateDailyReadingRequest {
    #[schema(value_type = Option<String>, example = "2026-11-11")]
    pub reading_date: Option<NaiveDate>,

    #[schema(value_type = Option<String>, example = "08:30:00")]
    pub reading_time: Option<String>,

    #[schema(value_type = f64, example = 750.0)]
    pub ppm: Decimal,

    #[schema(value_type = f64, example = 6.2)]
    pub ph: Decimal,

    #[schema(value_type = Option<f64>, example = 24.5)]
    pub temp_env: Option<Decimal>,

    #[schema(value_type = Option<f64>, example = 21.0)]
    pub temp_water: Option<Decimal>,

    #[schema(value_type = Option<f64>, example = 68.0)]
    pub humidity: Option<Decimal>,

    #[validate(range(min = 0, message = "Mortality count cannot be negative"))]
    #[schema(example = 1)]
    pub mortality_count: Option<i32>,

    #[validate(length(max = 500, message = "Notes cannot exceed 500 characters"))]
    #[schema(example = "Pertumbuhan daun seragam, tandon baru ditambah air")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct DailyReadingListQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub overall_status: Option<MetricStatus>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDailyReadingsHeader {
    pub id: String,
    pub session_name: String,
    pub plant_name: String,
    pub plant_variety: String,
    pub location_name: String,
    pub installation_name: Option<String>,
    #[schema(value_type = String, example = "2026-11-01")]
    pub start_date: NaiveDate,
    #[schema(value_type = String, example = "2026-12-06")]
    pub estimated_harvest_date: NaiveDate,
    pub total_holes: i32,
    #[schema(value_type = f64, example = 150.0)]
    pub target_yield_kg: Decimal,
    pub status: SessionStatus,
    pub is_locked: bool,
    pub growth_info: GrowthInfo,
    pub active_targets: ActiveMetricTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDailyReadingsResponse {
    pub session: SessionDailyReadingsHeader,
    pub total_daily_mortality: i64,
    pub total_readings: i64,
    pub latest_reading: Option<DailyReadingItem>,
    pub readings: Vec<DailyReadingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnreportedTodayResponse {
    #[schema(value_type = String, example = "2026-11-22")]
    pub evaluation_date: NaiveDate,
    pub total_active_sessions: i64,
    pub reported_count: i64,
    pub unreported_count: i64,
    pub unreported_sessions: Vec<SessionItemResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct UnreportedTodayQuery {
    pub date: Option<NaiveDate>,
    pub location_id: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DailyReadingEntity {
    pub id: String,
    pub session_id: String,
    pub reading_date: NaiveDate,
    pub reading_time: NaiveTime,
    pub ppm: Decimal,
    pub ph: Decimal,
    pub temp_env: Option<Decimal>,
    pub temp_water: Option<Decimal>,
    pub humidity: Option<Decimal>,
    pub mortality_count: i32,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}
