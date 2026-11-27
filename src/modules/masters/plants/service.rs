use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::masters::plants::dto::{
    CreatePlantRequest, PlantEntity, PlantListQuery, PlantResponse, PpmWeekRange,
    UpdatePlantRequest,
};
use crate::modules::masters::plants::repository::PlantRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedPpmRange {
    pub ideal_min: i32,
    pub ideal_max: i32,
    pub tol_min: i32,
    pub tol_max: i32,
}

pub fn calculate_veg_phase_days(veg_weeks_len: usize) -> i32 {
    if veg_weeks_len > 0 {
        (veg_weeks_len as i32) * 7
    } else {
        14 // Default 2 minggu jika jadwal kosong
    }
}

pub fn calculate_gen_phase_days(growth_est_days: i32, veg_phase_days: i32) -> i32 {
    (growth_est_days - veg_phase_days).max(0)
}

pub fn derive_ppm_range(
    weeks: &[PpmWeekRange],
    tol_min_override: Option<i32>,
    tol_max_override: Option<i32>,
) -> Result<DerivedPpmRange, AppError> {
    if weeks.is_empty() {
        return Ok(DerivedPpmRange {
            ideal_min: 0,
            ideal_max: 0,
            tol_min: tol_min_override.unwrap_or(0),
            tol_max: tol_max_override.unwrap_or(0),
        });
    }

    for (idx, w) in weeks.iter().enumerate() {
        if w.min < 0 || w.max < 0 {
            return Err(AppError::BadRequest(format!(
                "Nilai PPM pada minggu ke-{} tidak boleh negatif",
                idx + 1
            )));
        }
        if w.min > w.max {
            return Err(AppError::BadRequest(format!(
                "Nilai PPM minimum ({}) tidak boleh lebih besar dari maksimum ({}) pada minggu ke-{}",
                w.min,
                w.max,
                idx + 1
            )));
        }
    }

    let ideal_min = weeks.iter().map(|w| w.min).min().unwrap_or(0);
    let ideal_max = weeks.iter().map(|w| w.max).max().unwrap_or(0);

    let tol_min = match tol_min_override {
        Some(t) => {
            if t < 0 {
                return Err(AppError::BadRequest(
                    "Batas toleransi minimum PPM tidak boleh negatif".to_string(),
                ));
            }
            if t > ideal_min {
                return Err(AppError::BadRequest(
                    "Batas toleransi minimum PPM tidak boleh lebih besar dari ideal minimum"
                        .to_string(),
                ));
            }
            t
        }
        None => (ideal_min - 100).max(0),
    };

    let tol_max = match tol_max_override {
        Some(t) => {
            if t < ideal_max {
                return Err(AppError::BadRequest(
                    "Batas toleransi maksimum PPM tidak boleh lebih kecil dari ideal maksimum"
                        .to_string(),
                ));
            }
            t
        }
        None => ideal_max + 100,
    };

    Ok(DerivedPpmRange {
        ideal_min,
        ideal_max,
        tol_min,
        tol_max,
    })
}

pub fn validate_plant_boundaries(req: &CreatePlantRequest) -> Result<(), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Nama tanaman tidak boleh kosong atau hanya berisi spasi".to_string(),
        ));
    }

    if req.growth_est_days < 1 {
        return Err(AppError::BadRequest(
            "Estimasi hari panen minimal 1 hari".to_string(),
        ));
    }

    let zero = Decimal::ZERO;
    let max_yield = Decimal::from(100_000);
    if req.target_yield_per_hole <= zero || req.target_yield_per_hole > max_yield {
        return Err(AppError::BadRequest(
            "Target hasil per lubang harus berada dalam rentang 0.01 hingga 100,000.00 gram"
                .to_string(),
        ));
    }

    let fourteen = Decimal::from(14);
    let hundred = Decimal::from(100);

    if req.min_ph < zero || req.max_ph > fourteen {
        return Err(AppError::BadRequest(
            "Nilai pH harus berada dalam rentang 0.00 hingga 14.00".to_string(),
        ));
    }

    if req.min_ph > req.ideal_min_ph
        || req.ideal_min_ph > req.ideal_max_ph
        || req.ideal_max_ph > req.max_ph
    {
        return Err(AppError::BadRequest(
            "Urutan nilai pH tidak valid (harus min <= ideal_min <= ideal_max <= max)".to_string(),
        ));
    }

    let min_temp = Decimal::from(-50);
    let max_temp = Decimal::from(100);

    if req.ideal_min_temp_env < min_temp || req.ideal_max_temp_env > max_temp {
        return Err(AppError::BadRequest(
            "Suhu lingkungan harus berada dalam rentang -50.00 hingga 100.00 °C".to_string(),
        ));
    }

    if req.ideal_min_temp_env > req.ideal_max_temp_env {
        return Err(AppError::BadRequest(
            "Suhu lingkungan minimum tidak boleh lebih besar dari maksimum".to_string(),
        ));
    }

    if req.ideal_min_temp_water < min_temp || req.ideal_max_temp_water > max_temp {
        return Err(AppError::BadRequest(
            "Suhu air nutrisi harus berada dalam rentang -50.00 hingga 100.00 °C".to_string(),
        ));
    }

    if req.ideal_min_temp_water > req.ideal_max_temp_water {
        return Err(AppError::BadRequest(
            "Suhu air nutrisi minimum tidak boleh lebih besar dari maksimum".to_string(),
        ));
    }

    if req.ideal_min_hum < zero || req.ideal_max_hum > hundred {
        return Err(AppError::BadRequest(
            "Kelembapan udara harus berada dalam rentang 0.00% hingga 100.00%".to_string(),
        ));
    }

    if req.ideal_min_hum > req.ideal_max_hum {
        return Err(AppError::BadRequest(
            "Kelembapan udara minimum tidak boleh lebih besar dari maksimum".to_string(),
        ));
    }

    let max_ppm = 50_000;
    for week in req
        .ideal_ppm_veg_weeks
        .iter()
        .chain(req.ideal_ppm_gen_weeks.iter())
    {
        if week.min < 0 || week.min > max_ppm || week.max < 0 || week.max > max_ppm {
            return Err(AppError::BadRequest(
                "PPM mingguan harus berada dalam rentang 0 hingga 50,000".to_string(),
            ));
        }
    }
    if let Some(t) = req.tol_min_ppm_veg {
        if t < 0 || t > max_ppm {
            return Err(AppError::BadRequest(
                "Batas toleransi PPM harus antara 0 dan 50,000".to_string(),
            ));
        }
    }
    if let Some(t) = req.tol_max_ppm_veg {
        if t < 0 || t > max_ppm {
            return Err(AppError::BadRequest(
                "Batas toleransi PPM harus antara 0 dan 50,000".to_string(),
            ));
        }
    }
    if let Some(t) = req.tol_min_ppm_gen {
        if t < 0 || t > max_ppm {
            return Err(AppError::BadRequest(
                "Batas toleransi PPM harus antara 0 dan 50,000".to_string(),
            ));
        }
    }
    if let Some(t) = req.tol_max_ppm_gen {
        if t < 0 || t > max_ppm {
            return Err(AppError::BadRequest(
                "Batas toleransi PPM harus antara 0 dan 50,000".to_string(),
            ));
        }
    }

    Ok(())
}

fn map_entity_to_response(e: PlantEntity) -> PlantResponse {
    let gen_phase = calculate_gen_phase_days(e.growth_est_days, e.veg_phase_days);
    PlantResponse {
        id: e.id,
        name: e.name,
        variety: e.variety,
        growth_est_days: e.growth_est_days,
        veg_phase_days: e.veg_phase_days,
        gen_phase_days: gen_phase,
        target_yield_per_hole: e.target_yield_per_hole,
        min_ph: e.min_ph,
        ideal_min_ph: e.ideal_min_ph,
        ideal_max_ph: e.ideal_max_ph,
        max_ph: e.max_ph,
        ideal_ppm_veg_weeks: e.ideal_ppm_veg_weeks.0,
        ideal_ppm_gen_weeks: e.ideal_ppm_gen_weeks.0,
        ideal_min_ppm_veg: e.ideal_min_ppm_veg,
        ideal_max_ppm_veg: e.ideal_max_ppm_veg,
        tol_min_ppm_veg: e.tol_min_ppm_veg,
        tol_max_ppm_veg: e.tol_max_ppm_veg,
        ideal_min_ppm_gen: e.ideal_min_ppm_gen,
        ideal_max_ppm_gen: e.ideal_max_ppm_gen,
        tol_min_ppm_gen: e.tol_min_ppm_gen,
        tol_max_ppm_gen: e.tol_max_ppm_gen,
        ideal_min_temp_env: e.ideal_min_temp_env,
        ideal_max_temp_env: e.ideal_max_temp_env,
        ideal_min_temp_water: e.ideal_min_temp_water,
        ideal_max_temp_water: e.ideal_max_temp_water,
        ideal_min_hum: e.ideal_min_hum,
        ideal_max_hum: e.ideal_max_hum,
        is_active: e.is_active,
        created_at: e.created_at,
        created_by: e.created_by,
        modify_at: e.modify_at,
        modify_by: e.modify_by,
    }
}

#[async_trait]
pub trait PlantService: Send + Sync {
    async fn list_plants(&self, query: PlantListQuery) -> Result<Vec<PlantResponse>, AppError>;
    async fn get_plant(&self, id: &str) -> Result<PlantResponse, AppError>;
    async fn create_plant(
        &self,
        req: CreatePlantRequest,
        actor: &str,
    ) -> Result<PlantResponse, AppError>;
    async fn update_plant(
        &self,
        id: &str,
        req: UpdatePlantRequest,
        actor: &str,
    ) -> Result<PlantResponse, AppError>;
    async fn delete_plant(&self, id: &str) -> Result<(), AppError>;
}

pub struct PlantServiceImpl {
    repo: Arc<dyn PlantRepository>,
}

impl PlantServiceImpl {
    pub fn new(repo: Arc<dyn PlantRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl PlantService for PlantServiceImpl {
    async fn list_plants(&self, query: PlantListQuery) -> Result<Vec<PlantResponse>, AppError> {
        let entities = self.repo.find_all(&query).await?;
        Ok(entities.into_iter().map(map_entity_to_response).collect())
    }

    async fn get_plant(&self, id: &str) -> Result<PlantResponse, AppError> {
        let entity = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Plant with ID '{id}' not found")))?;
        Ok(map_entity_to_response(entity))
    }

    async fn create_plant(
        &self,
        req: CreatePlantRequest,
        actor: &str,
    ) -> Result<PlantResponse, AppError> {
        req.validate()?;
        validate_plant_boundaries(&req)?;

        let variety = req.variety.unwrap_or_default().trim().to_string();
        let name = req.name.trim().to_string();

        if let Some(existing) = self.repo.find_by_name_and_variety(&name, &variety).await? {
            return Err(AppError::Conflict(format!(
                "Tanaman dengan nama '{}' dan varietas '{}' sudah terdaftar",
                existing.name, existing.variety
            )));
        }

        let veg_phase_days = calculate_veg_phase_days(req.ideal_ppm_veg_weeks.len());
        let veg_ppm = derive_ppm_range(
            &req.ideal_ppm_veg_weeks,
            req.tol_min_ppm_veg,
            req.tol_max_ppm_veg,
        )?;
        let gen_ppm = derive_ppm_range(
            &req.ideal_ppm_gen_weeks,
            req.tol_min_ppm_gen,
            req.tol_max_ppm_gen,
        )?;

        let now = Utc::now();
        let plant_id = format!("plt_{}", Uuid::new_v4());

        let entity = PlantEntity {
            id: plant_id,
            name,
            variety,
            growth_est_days: req.growth_est_days,
            veg_phase_days,
            target_yield_per_hole: req.target_yield_per_hole,
            min_ph: req.min_ph,
            ideal_min_ph: req.ideal_min_ph,
            ideal_max_ph: req.ideal_max_ph,
            max_ph: req.max_ph,
            ideal_ppm_veg_weeks: sqlx::types::Json(req.ideal_ppm_veg_weeks),
            ideal_ppm_gen_weeks: sqlx::types::Json(req.ideal_ppm_gen_weeks),
            ideal_min_ppm_veg: veg_ppm.ideal_min,
            ideal_max_ppm_veg: veg_ppm.ideal_max,
            tol_min_ppm_veg: veg_ppm.tol_min,
            tol_max_ppm_veg: veg_ppm.tol_max,
            ideal_min_ppm_gen: gen_ppm.ideal_min,
            ideal_max_ppm_gen: gen_ppm.ideal_max,
            tol_min_ppm_gen: gen_ppm.tol_min,
            tol_max_ppm_gen: gen_ppm.tol_max,
            ideal_min_temp_env: req.ideal_min_temp_env,
            ideal_max_temp_env: req.ideal_max_temp_env,
            ideal_min_temp_water: req.ideal_min_temp_water,
            ideal_max_temp_water: req.ideal_max_temp_water,
            ideal_min_hum: req.ideal_min_hum,
            ideal_max_hum: req.ideal_max_hum,
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repo.create(&entity).await?;
        Ok(map_entity_to_response(created))
    }

    async fn update_plant(
        &self,
        id: &str,
        req: UpdatePlantRequest,
        actor: &str,
    ) -> Result<PlantResponse, AppError> {
        req.validate()?;
        validate_plant_boundaries(&req)?;

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Plant with ID '{id}' not found")))?;

        let variety = req.variety.unwrap_or_default().trim().to_string();
        let name = req.name.trim().to_string();

        if let Some(conflict) = self.repo.find_by_name_and_variety(&name, &variety).await? {
            if conflict.id != id {
                return Err(AppError::Conflict(format!(
                    "Tanaman dengan nama '{}' dan varietas '{}' sudah terdaftar",
                    conflict.name, conflict.variety
                )));
            }
        }

        let veg_phase_days = calculate_veg_phase_days(req.ideal_ppm_veg_weeks.len());
        let veg_ppm = derive_ppm_range(
            &req.ideal_ppm_veg_weeks,
            req.tol_min_ppm_veg,
            req.tol_max_ppm_veg,
        )?;
        let gen_ppm = derive_ppm_range(
            &req.ideal_ppm_gen_weeks,
            req.tol_min_ppm_gen,
            req.tol_max_ppm_gen,
        )?;

        let entity = PlantEntity {
            id: id.to_string(),
            name,
            variety,
            growth_est_days: req.growth_est_days,
            veg_phase_days,
            target_yield_per_hole: req.target_yield_per_hole,
            min_ph: req.min_ph,
            ideal_min_ph: req.ideal_min_ph,
            ideal_max_ph: req.ideal_max_ph,
            max_ph: req.max_ph,
            ideal_ppm_veg_weeks: sqlx::types::Json(req.ideal_ppm_veg_weeks),
            ideal_ppm_gen_weeks: sqlx::types::Json(req.ideal_ppm_gen_weeks),
            ideal_min_ppm_veg: veg_ppm.ideal_min,
            ideal_max_ppm_veg: veg_ppm.ideal_max,
            tol_min_ppm_veg: veg_ppm.tol_min,
            tol_max_ppm_veg: veg_ppm.tol_max,
            ideal_min_ppm_gen: gen_ppm.ideal_min,
            ideal_max_ppm_gen: gen_ppm.ideal_max,
            tol_min_ppm_gen: gen_ppm.tol_min,
            tol_max_ppm_gen: gen_ppm.tol_max,
            ideal_min_temp_env: req.ideal_min_temp_env,
            ideal_max_temp_env: req.ideal_max_temp_env,
            ideal_min_temp_water: req.ideal_min_temp_water,
            ideal_max_temp_water: req.ideal_max_temp_water,
            ideal_min_hum: req.ideal_min_hum,
            ideal_max_hum: req.ideal_max_hum,
            is_active: req.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: Utc::now(),
            modify_by: Some(actor.to_string()),
        };

        let updated = self.repo.update(&entity).await?;
        Ok(map_entity_to_response(updated))
    }

    async fn delete_plant(&self, id: &str) -> Result<(), AppError> {
        let _ = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Plant with ID '{id}' not found")))?;

        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plant_phase_and_ppm_derivation_sim_1() {
        // Simulasi 1 dari PRD Section 3.5:
        // - growth_est_days: 35
        // - veg_weeks: 3 minggu (560-700, 700-840, 800-900)
        // - gen_weeks: 2 minggu (900-1050, 1000-1150)
        let veg_weeks = vec![
            PpmWeekRange { min: 560, max: 700 },
            PpmWeekRange { min: 700, max: 840 },
            PpmWeekRange { min: 800, max: 900 },
        ];
        let gen_weeks = vec![
            PpmWeekRange {
                min: 900,
                max: 1050,
            },
            PpmWeekRange {
                min: 1000,
                max: 1150,
            },
        ];

        let veg_phase = calculate_veg_phase_days(veg_weeks.len());
        assert_eq!(veg_phase, 21);

        let gen_phase = calculate_gen_phase_days(35, veg_phase);
        assert_eq!(gen_phase, 14);

        let veg_ppm = derive_ppm_range(&veg_weeks, None, None).expect("Veg PPM should succeed");
        assert_eq!(veg_ppm.ideal_min, 560);
        assert_eq!(veg_ppm.ideal_max, 900);
        assert_eq!(veg_ppm.tol_min, 460);
        assert_eq!(veg_ppm.tol_max, 1000);

        let gen_ppm = derive_ppm_range(&gen_weeks, None, None).expect("Gen PPM should succeed");
        assert_eq!(gen_ppm.ideal_min, 900);
        assert_eq!(gen_ppm.ideal_max, 1150);
        assert_eq!(gen_ppm.tol_min, 800);
        assert_eq!(gen_ppm.tol_max, 1250);
    }

    #[test]
    fn test_plant_boundary_validation_invariants() {
        let make_req = |days: i32,
                        target_yield: &str,
                        min_ph: &str,
                        ideal_min_ph: &str,
                        ideal_max_ph: &str,
                        max_ph: &str| {
            use std::str::FromStr;
            CreatePlantRequest {
                name: "Selada".to_string(),
                variety: None,
                growth_est_days: days,
                target_yield_per_hole: Decimal::from_str(target_yield).unwrap(),
                min_ph: Decimal::from_str(min_ph).unwrap(),
                ideal_min_ph: Decimal::from_str(ideal_min_ph).unwrap(),
                ideal_max_ph: Decimal::from_str(ideal_max_ph).unwrap(),
                max_ph: Decimal::from_str(max_ph).unwrap(),
                ideal_ppm_veg_weeks: vec![],
                ideal_ppm_gen_weeks: vec![],
                tol_min_ppm_veg: None,
                tol_max_ppm_veg: None,
                tol_min_ppm_gen: None,
                tol_max_ppm_gen: None,
                ideal_min_temp_env: Decimal::from(20),
                ideal_max_temp_env: Decimal::from(25),
                ideal_min_temp_water: Decimal::from(18),
                ideal_max_temp_water: Decimal::from(22),
                ideal_min_hum: Decimal::from(60),
                ideal_max_hum: Decimal::from(80),
                is_active: Some(true),
            }
        };

        // Min pH > Max pH -> Harus ditolak
        let req_ph = make_req(35, "250.0", "6.8", "6.0", "6.5", "5.5");
        assert!(matches!(
            validate_plant_boundaries(&req_ph),
            Err(AppError::BadRequest(_))
        ));

        // Growth est days < 1 -> Harus ditolak
        let req_days = make_req(0, "250.0", "5.0", "5.5", "6.5", "7.0");
        assert!(matches!(
            validate_plant_boundaries(&req_days),
            Err(AppError::BadRequest(_))
        ));

        // Target yield <= 0 -> Harus ditolak
        let req_yield = make_req(30, "0.0", "5.0", "5.5", "6.5", "7.0");
        assert!(matches!(
            validate_plant_boundaries(&req_yield),
            Err(AppError::BadRequest(_))
        ));
    }
}
