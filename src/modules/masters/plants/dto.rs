use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PpmWeekRange {
    #[schema(example = 560)]
    pub min: i32,
    #[schema(example = 700)]
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlantEntity {
    pub id: String,
    pub name: String,
    pub variety: String,
    pub growth_est_days: i32,
    pub veg_phase_days: i32,
    pub target_yield_per_hole: Decimal,
    pub min_ph: Decimal,
    pub ideal_min_ph: Decimal,
    pub ideal_max_ph: Decimal,
    pub max_ph: Decimal,
    pub ideal_ppm_veg_weeks: sqlx::types::Json<Vec<PpmWeekRange>>,
    pub ideal_ppm_gen_weeks: sqlx::types::Json<Vec<PpmWeekRange>>,
    pub ideal_min_ppm_veg: i32,
    pub ideal_max_ppm_veg: i32,
    pub tol_min_ppm_veg: i32,
    pub tol_max_ppm_veg: i32,
    pub ideal_min_ppm_gen: i32,
    pub ideal_max_ppm_gen: i32,
    pub tol_min_ppm_gen: i32,
    pub tol_max_ppm_gen: i32,
    pub ideal_min_temp_env: Decimal,
    pub ideal_max_temp_env: Decimal,
    pub ideal_min_temp_water: Decimal,
    pub ideal_max_temp_water: Decimal,
    pub ideal_min_hum: Decimal,
    pub ideal_max_hum: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePlantRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    #[schema(example = "Selada Romaine")]
    pub name: String,

    #[validate(length(max = 100, message = "Variety cannot exceed 100 characters"))]
    #[schema(example = "Green Towers")]
    pub variety: Option<String>,

    #[schema(example = 35)]
    pub growth_est_days: i32,

    #[schema(value_type = f64, example = 250.0)]
    pub target_yield_per_hole: Decimal,

    #[schema(value_type = f64, example = 5.0)]
    pub min_ph: Decimal,

    #[schema(value_type = f64, example = 5.5)]
    pub ideal_min_ph: Decimal,

    #[schema(value_type = f64, example = 6.5)]
    pub ideal_max_ph: Decimal,

    #[schema(value_type = f64, example = 7.0)]
    pub max_ph: Decimal,

    #[schema(example = json!([{"min": 560, "max": 700}, {"min": 700, "max": 840}, {"min": 800, "max": 900}]))]
    pub ideal_ppm_veg_weeks: Vec<PpmWeekRange>,

    #[schema(example = json!([{"min": 900, "max": 1050}, {"min": 1000, "max": 1150}]))]
    pub ideal_ppm_gen_weeks: Vec<PpmWeekRange>,

    pub tol_min_ppm_veg: Option<i32>,
    pub tol_max_ppm_veg: Option<i32>,
    pub tol_min_ppm_gen: Option<i32>,
    pub tol_max_ppm_gen: Option<i32>,

    #[schema(value_type = f64, example = 18.0)]
    pub ideal_min_temp_env: Decimal,

    #[schema(value_type = f64, example = 26.0)]
    pub ideal_max_temp_env: Decimal,

    #[schema(value_type = f64, example = 18.0)]
    pub ideal_min_temp_water: Decimal,

    #[schema(value_type = f64, example = 24.0)]
    pub ideal_max_temp_water: Decimal,

    #[schema(value_type = f64, example = 60.0)]
    pub ideal_min_hum: Decimal,

    #[schema(value_type = f64, example = 80.0)]
    pub ideal_max_hum: Decimal,

    pub is_active: Option<bool>,
}

pub type UpdatePlantRequest = CreatePlantRequest;

#[derive(Debug, Deserialize, IntoParams)]
pub struct PlantListQuery {
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlantResponse {
    pub id: String,
    pub name: String,
    pub variety: String,
    pub growth_est_days: i32,
    pub veg_phase_days: i32,
    pub gen_phase_days: i32,
    #[schema(value_type = f64, example = 250.0)]
    pub target_yield_per_hole: Decimal,
    #[schema(value_type = f64, example = 5.0)]
    pub min_ph: Decimal,
    #[schema(value_type = f64, example = 5.5)]
    pub ideal_min_ph: Decimal,
    #[schema(value_type = f64, example = 6.5)]
    pub ideal_max_ph: Decimal,
    #[schema(value_type = f64, example = 7.0)]
    pub max_ph: Decimal,
    pub ideal_ppm_veg_weeks: Vec<PpmWeekRange>,
    pub ideal_ppm_gen_weeks: Vec<PpmWeekRange>,
    pub ideal_min_ppm_veg: i32,
    pub ideal_max_ppm_veg: i32,
    pub tol_min_ppm_veg: i32,
    pub tol_max_ppm_veg: i32,
    pub ideal_min_ppm_gen: i32,
    pub ideal_max_ppm_gen: i32,
    pub tol_min_ppm_gen: i32,
    pub tol_max_ppm_gen: i32,
    #[schema(value_type = f64, example = 18.0)]
    pub ideal_min_temp_env: Decimal,
    #[schema(value_type = f64, example = 26.0)]
    pub ideal_max_temp_env: Decimal,
    #[schema(value_type = f64, example = 18.0)]
    pub ideal_min_temp_water: Decimal,
    #[schema(value_type = f64, example = 24.0)]
    pub ideal_max_temp_water: Decimal,
    #[schema(value_type = f64, example = 60.0)]
    pub ideal_min_hum: Decimal,
    #[schema(value_type = f64, example = 80.0)]
    pub ideal_max_hum: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub modify_at: DateTime<Utc>,
    pub modify_by: Option<String>,
}
