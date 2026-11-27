use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::modules::daily_readings::dto::{ActiveMetricTargets, MetricStatus};
use crate::modules::masters::plants::dto::PpmWeekRange;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Completed => write!(f, "Completed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl FromStr for SessionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(Self::Active),
            "Completed" => Ok(Self::Completed),
            "Cancelled" => Ok(Self::Cancelled),
            other => Err(format!("Unknown session status: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GrowthPhase {
    Semai,
    Vegetatif,
    Generatif,
}

impl fmt::Display for GrowthPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Semai => write!(f, "SEMAI"),
            Self::Vegetatif => write!(f, "VEGETATIF"),
            Self::Generatif => write!(f, "GENERATIF"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GrowthInfo {
    pub phase: GrowthPhase,
    pub hst: i64,
    pub seeding_days: i64,
    pub active_week: i32,
    pub week_index: usize,
    #[schema(value_type = f64, example = 28.57)]
    pub progress_percent: Decimal,
    pub is_overdue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LatestReadingSummary {
    pub id: String,
    #[schema(value_type = String, example = "2026-11-11")]
    pub reading_date: NaiveDate,
    #[schema(value_type = String, example = "08:30:00")]
    pub reading_time: String,
    #[schema(value_type = f64, example = 750.0)]
    pub ppm: Decimal,
    #[schema(value_type = f64, example = 6.2)]
    pub ph: Decimal,
    pub overall_status: MetricStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionKpi {
    pub active_sessions: i64,
    pub active_total_holes: i64,
    #[schema(value_type = f64, example = 300.0)]
    pub active_estimated_yield_kg: Decimal,
    pub completed_sessions: i64,
    pub unreported_today_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionItemResponse {
    pub id: String,
    pub session_name: String,
    pub plant_type_id: String,
    pub plant_name: String,
    pub plant_variety: String,
    pub location_id: String,
    pub location_name: String,
    pub installation_id: Option<String>,
    pub installation_name: Option<String>,
    #[schema(value_type = Option<String>, example = "2026-10-20")]
    pub seeding_date: Option<NaiveDate>,
    #[schema(value_type = String, example = "2026-11-01")]
    pub start_date: NaiveDate,
    #[schema(value_type = String, example = "2026-12-06")]
    pub estimated_harvest_date: NaiveDate,
    #[schema(value_type = Option<String>, example = "2026-12-06")]
    pub harvest_date: Option<NaiveDate>,
    pub total_holes: i32,
    pub status: SessionStatus,
    pub is_locked: bool,
    #[schema(value_type = f64, example = 100.0)]
    pub target_yield_kg: Decimal,
    #[schema(value_type = Option<f64>, example = 92.5)]
    pub actual_yield_kg: Option<Decimal>,
    pub saleable_count: Option<i32>,
    pub mortality_count: i32,
    pub total_daily_mortality: i64,
    #[schema(value_type = Option<f64>, example = 92.5)]
    pub yield_achievement_percent: Option<Decimal>,
    #[schema(value_type = f64, example = 98.5)]
    pub survival_rate_percent: Decimal,
    pub growth_info: GrowthInfo,
    pub has_reported_today: bool,
    pub latest_reading: Option<LatestReadingSummary>,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionTargetSnapshot {
    pub target_growth_est_days: i32,
    pub target_veg_phase_days: i32,
    #[schema(value_type = f64, example = 250.0)]
    pub target_yield_per_hole: Decimal,
    #[schema(value_type = f64, example = 5.0)]
    pub target_min_ph: Decimal,
    #[schema(value_type = f64, example = 5.5)]
    pub target_ideal_min_ph: Decimal,
    #[schema(value_type = f64, example = 6.5)]
    pub target_ideal_max_ph: Decimal,
    #[schema(value_type = f64, example = 7.0)]
    pub target_max_ph: Decimal,
    pub target_ppm_veg_weeks: Vec<PpmWeekRange>,
    pub target_ppm_gen_weeks: Vec<PpmWeekRange>,
    pub target_ideal_min_ppm_veg: i32,
    pub target_ideal_max_ppm_veg: i32,
    pub target_tol_min_ppm_veg: i32,
    pub target_tol_max_ppm_veg: i32,
    pub target_ideal_min_ppm_gen: i32,
    pub target_ideal_max_ppm_gen: i32,
    pub target_tol_min_ppm_gen: i32,
    pub target_tol_max_ppm_gen: i32,
    #[schema(value_type = f64, example = 18.0)]
    pub target_ideal_min_temp_env: Decimal,
    #[schema(value_type = f64, example = 26.0)]
    pub target_ideal_max_temp_env: Decimal,
    #[schema(value_type = f64, example = 18.0)]
    pub target_ideal_min_temp_water: Decimal,
    #[schema(value_type = f64, example = 24.0)]
    pub target_ideal_max_temp_water: Decimal,
    #[schema(value_type = f64, example = 60.0)]
    pub target_ideal_min_hum: Decimal,
    #[schema(value_type = f64, example = 80.0)]
    pub target_ideal_max_hum: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDetailResponse {
    #[serde(flatten)]
    pub item: SessionItemResponse,
    pub target_snapshot: SessionTargetSnapshot,
    pub active_targets: ActiveMetricTargets,
    pub harvest_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionListResponse {
    pub kpi: SessionKpi,
    pub items: Vec<SessionItemResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct SessionListQuery {
    pub location_id: Option<String>,
    pub installation_id: Option<String>,
    pub plant_type_id: Option<String>,
    pub status: Option<SessionStatus>,
    pub search: Option<String>,
    pub date: Option<NaiveDate>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateSessionRequest {
    #[validate(length(
        min = 1,
        max = 150,
        message = "Session name must be between 1 and 150 characters"
    ))]
    #[schema(example = "Batch #12 - Selada Romaine A1")]
    pub session_name: String,

    #[validate(length(min = 1, message = "Plant type ID is required"))]
    #[schema(example = "plt_550e8400-e29b-41d4-a716-446655440000")]
    pub plant_type_id: String,

    #[validate(length(min = 1, message = "Location ID is required"))]
    #[schema(example = "loc_11111111-2222-4333-8444-555555555555")]
    pub location_id: String,

    #[schema(example = "ins_9f8b7c6d-1234-4a5b-8c9d-0e1f2a3b4c5d")]
    pub installation_id: Option<String>,

    #[schema(value_type = Option<String>, example = "2026-10-20")]
    pub seeding_date: Option<NaiveDate>,

    #[schema(value_type = String, example = "2026-11-01")]
    pub start_date: NaiveDate,

    #[schema(value_type = Option<String>, example = "2026-12-06")]
    pub estimated_harvest_date: Option<NaiveDate>,

    #[validate(range(min = 1, message = "Total holes must be at least 1"))]
    #[schema(example = 400)]
    pub total_holes: i32,

    #[schema(example = "Batch percobaan awal November")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateSessionRequest {
    #[validate(length(
        min = 1,
        max = 150,
        message = "Session name must be between 1 and 150 characters"
    ))]
    #[schema(example = "Batch #12 - Selada Romaine A1")]
    pub session_name: String,

    #[validate(length(min = 1, message = "Location ID is required"))]
    #[schema(example = "loc_11111111-2222-4333-8444-555555555555")]
    pub location_id: String,

    #[schema(example = "ins_9f8b7c6d-1234-4a5b-8c9d-0e1f2a3b4c5d")]
    pub installation_id: Option<String>,

    #[schema(value_type = Option<String>, example = "2026-10-20")]
    pub seeding_date: Option<NaiveDate>,

    #[schema(value_type = String, example = "2026-11-01")]
    pub start_date: NaiveDate,

    #[schema(value_type = Option<String>, example = "2026-12-06")]
    pub estimated_harvest_date: Option<NaiveDate>,

    #[validate(range(min = 1, message = "Total holes must be at least 1"))]
    #[schema(example = 400)]
    pub total_holes: i32,

    pub status: Option<SessionStatus>,

    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CompleteHarvestRequest {
    #[schema(value_type = Option<String>, example = "2026-12-06")]
    pub harvest_date: Option<NaiveDate>,

    #[schema(value_type = f64, example = 92.5)]
    pub actual_yield_kg: Decimal,

    #[validate(range(min = 0, message = "Saleable count cannot be negative"))]
    #[schema(example = 370)]
    pub saleable_count: i32,

    #[validate(range(min = 0, message = "Mortality count cannot be negative"))]
    #[schema(example = 24)]
    pub mortality_count: i32,

    #[schema(example = "Panen serentak pagi hari, kualitas grade A 92%")]
    pub harvest_notes: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionEntity {
    pub id: String,
    pub session_name: String,
    pub plant_type_id: String,
    pub location_id: String,
    pub installation_id: Option<String>,

    pub seeding_date: Option<NaiveDate>,
    pub start_date: NaiveDate,
    pub estimated_harvest_date: NaiveDate,
    pub harvest_date: Option<NaiveDate>,

    pub total_holes: i32,
    pub status: String,
    pub is_locked: bool,

    pub target_growth_est_days: i32,
    pub target_veg_phase_days: i32,
    pub target_yield_per_hole: Decimal,

    pub target_min_ph: Decimal,
    pub target_ideal_min_ph: Decimal,
    pub target_ideal_max_ph: Decimal,
    pub target_max_ph: Decimal,

    pub target_ppm_veg_weeks: sqlx::types::Json<Vec<PpmWeekRange>>,
    pub target_ppm_gen_weeks: sqlx::types::Json<Vec<PpmWeekRange>>,
    pub target_ideal_min_ppm_veg: i32,
    pub target_ideal_max_ppm_veg: i32,
    pub target_tol_min_ppm_veg: i32,
    pub target_tol_max_ppm_veg: i32,
    pub target_ideal_min_ppm_gen: i32,
    pub target_ideal_max_ppm_gen: i32,
    pub target_tol_min_ppm_gen: i32,
    pub target_tol_max_ppm_gen: i32,

    pub target_ideal_min_temp_env: Decimal,
    pub target_ideal_max_temp_env: Decimal,
    pub target_ideal_min_temp_water: Decimal,
    pub target_ideal_max_temp_water: Decimal,
    pub target_ideal_min_hum: Decimal,
    pub target_ideal_max_hum: Decimal,

    pub actual_yield_kg: Option<Decimal>,
    pub saleable_count: Option<i32>,
    pub mortality_count: i32,
    pub harvest_notes: String,

    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRowWithRelations {
    pub id: String,
    pub session_name: String,
    pub plant_type_id: String,
    pub plant_name: String,
    pub plant_variety: String,
    pub location_id: String,
    pub location_name: String,
    pub installation_id: Option<String>,
    pub installation_name: Option<String>,

    pub seeding_date: Option<NaiveDate>,
    pub start_date: NaiveDate,
    pub estimated_harvest_date: NaiveDate,
    pub harvest_date: Option<NaiveDate>,

    pub total_holes: i32,
    pub status: String,
    pub is_locked: bool,

    pub target_growth_est_days: i32,
    pub target_veg_phase_days: i32,
    pub target_yield_per_hole: Decimal,

    pub target_min_ph: Decimal,
    pub target_ideal_min_ph: Decimal,
    pub target_ideal_max_ph: Decimal,
    pub target_max_ph: Decimal,

    pub target_ppm_veg_weeks: sqlx::types::Json<Vec<PpmWeekRange>>,
    pub target_ppm_gen_weeks: sqlx::types::Json<Vec<PpmWeekRange>>,
    pub target_ideal_min_ppm_veg: i32,
    pub target_ideal_max_ppm_veg: i32,
    pub target_tol_min_ppm_veg: i32,
    pub target_tol_max_ppm_veg: i32,
    pub target_ideal_min_ppm_gen: i32,
    pub target_ideal_max_ppm_gen: i32,
    pub target_tol_min_ppm_gen: i32,
    pub target_tol_max_ppm_gen: i32,

    pub target_ideal_min_temp_env: Decimal,
    pub target_ideal_max_temp_env: Decimal,
    pub target_ideal_min_temp_water: Decimal,
    pub target_ideal_max_temp_water: Decimal,
    pub target_ideal_min_hum: Decimal,
    pub target_ideal_max_hum: Decimal,

    pub actual_yield_kg: Option<Decimal>,
    pub saleable_count: Option<i32>,
    pub mortality_count: i32,
    pub harvest_notes: String,

    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,

    pub total_daily_mortality: Option<i64>,
    pub has_reported_today: Option<bool>,

    // Latest reading joined fields
    pub latest_reading_id: Option<String>,
    pub latest_reading_date: Option<NaiveDate>,
    pub latest_reading_time: Option<chrono::NaiveTime>,
    pub latest_reading_ppm: Option<Decimal>,
    pub latest_reading_ph: Option<Decimal>,
}
