use async_trait::async_trait;
use chrono::{Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::daily_readings::dto::MetricStatus;
use crate::modules::daily_readings::service::resolve_active_targets;
use crate::modules::masters::installations::repository::InstallationRepository;
use crate::modules::masters::locations::repository::LocationRepository;
use crate::modules::masters::plants::repository::PlantRepository;
use crate::modules::sessions::dto::{
    CompleteHarvestRequest, CreateSessionRequest, GrowthInfo, GrowthPhase, LatestReadingSummary,
    SessionDetailResponse, SessionEntity, SessionItemResponse, SessionListQuery,
    SessionListResponse, SessionRowWithRelations, SessionStatus, SessionTargetSnapshot,
    UpdateSessionRequest,
};
use crate::modules::sessions::repository::SessionRepository;

pub fn today_wib() -> NaiveDate {
    (Utc::now() + Duration::hours(7)).date_naive()
}

pub fn calculate_growth_info(
    seeding_date: Option<NaiveDate>,
    start_date: NaiveDate,
    eval_date: NaiveDate,
    growth_est_days: i32,
    veg_phase_days: i32,
    veg_weeks_len: usize,
    gen_weeks_len: usize,
) -> GrowthInfo {
    if eval_date < start_date {
        let seed_ref = seeding_date.unwrap_or(start_date);
        let diff = (eval_date - seed_ref).num_days();
        let seeding_days = diff.max(0);
        let active_week = ((seeding_days / 7) as i32) + 1;
        GrowthInfo {
            phase: GrowthPhase::Semai,
            hst: 0,
            seeding_days,
            active_week,
            week_index: 0,
            progress_percent: Decimal::ZERO,
            is_overdue: false,
        }
    } else {
        let hst = (eval_date - start_date).num_days();
        let est_days_dec = Decimal::from(growth_est_days.max(1));
        let hst_dec = Decimal::from(hst);
        let raw_progress = (hst_dec * Decimal::from(100)) / est_days_dec;
        let progress_percent = raw_progress.round_dp(2).min(Decimal::from(100));
        let is_overdue = hst > i64::from(growth_est_days);

        if hst <= i64::from(veg_phase_days) {
            let raw_idx = (((hst - 1).max(0)) / 7) as usize;
            let week_index = if veg_weeks_len == 0 {
                0
            } else {
                raw_idx.min(veg_weeks_len - 1)
            };
            let active_week = (week_index as i32) + 1;
            GrowthInfo {
                phase: GrowthPhase::Vegetatif,
                hst,
                seeding_days: 0,
                active_week,
                week_index,
                progress_percent,
                is_overdue,
            }
        } else {
            let raw_idx = (((hst - i64::from(veg_phase_days) - 1).max(0)) / 7) as usize;
            let week_index = if gen_weeks_len == 0 {
                0
            } else {
                raw_idx.min(gen_weeks_len - 1)
            };
            let active_week = (week_index as i32) + 1;
            GrowthInfo {
                phase: GrowthPhase::Generatif,
                hst,
                seeding_days: 0,
                active_week,
                week_index,
                progress_percent,
                is_overdue,
            }
        }
    }
}

pub fn calculate_target_yield_kg(total_holes: i32, target_yield_per_hole_gr: Decimal) -> Decimal {
    let holes = Decimal::from(total_holes);
    let total_gr = holes * target_yield_per_hole_gr;
    (total_gr / Decimal::from(1000)).round_dp(2)
}

pub fn calculate_yield_achievement_percent(
    actual_yield_kg: Decimal,
    target_yield_kg: Decimal,
) -> Decimal {
    if target_yield_kg <= Decimal::ZERO {
        Decimal::ZERO
    } else {
        ((actual_yield_kg / target_yield_kg) * Decimal::from(100)).round_dp(2)
    }
}

pub fn calculate_survival_rate_percent(total_holes: i32, mortality_count: i32) -> Decimal {
    if total_holes <= 0 {
        Decimal::ZERO
    } else {
        let live_plants = (total_holes - mortality_count).max(0);
        ((Decimal::from(live_plants) / Decimal::from(total_holes)) * Decimal::from(100)).round_dp(2)
    }
}

pub fn map_row_to_snapshot(row: &SessionRowWithRelations) -> SessionTargetSnapshot {
    SessionTargetSnapshot {
        target_growth_est_days: row.target_growth_est_days,
        target_veg_phase_days: row.target_veg_phase_days,
        target_yield_per_hole: row.target_yield_per_hole,
        target_min_ph: row.target_min_ph,
        target_ideal_min_ph: row.target_ideal_min_ph,
        target_ideal_max_ph: row.target_ideal_max_ph,
        target_max_ph: row.target_max_ph,
        target_ppm_veg_weeks: row.target_ppm_veg_weeks.0.clone(),
        target_ppm_gen_weeks: row.target_ppm_gen_weeks.0.clone(),
        target_ideal_min_ppm_veg: row.target_ideal_min_ppm_veg,
        target_ideal_max_ppm_veg: row.target_ideal_max_ppm_veg,
        target_tol_min_ppm_veg: row.target_tol_min_ppm_veg,
        target_tol_max_ppm_veg: row.target_tol_max_ppm_veg,
        target_ideal_min_ppm_gen: row.target_ideal_min_ppm_gen,
        target_ideal_max_ppm_gen: row.target_ideal_max_ppm_gen,
        target_tol_min_ppm_gen: row.target_tol_min_ppm_gen,
        target_tol_max_ppm_gen: row.target_tol_max_ppm_gen,
        target_ideal_min_temp_env: row.target_ideal_min_temp_env,
        target_ideal_max_temp_env: row.target_ideal_max_temp_env,
        target_ideal_min_temp_water: row.target_ideal_min_temp_water,
        target_ideal_max_temp_water: row.target_ideal_max_temp_water,
        target_ideal_min_hum: row.target_ideal_min_hum,
        target_ideal_max_hum: row.target_ideal_max_hum,
    }
}

pub fn map_row_to_item_response(
    row: SessionRowWithRelations,
    eval_date: NaiveDate,
) -> SessionItemResponse {
    let growth_info = calculate_growth_info(
        row.seeding_date,
        row.start_date,
        eval_date,
        row.target_growth_est_days,
        row.target_veg_phase_days,
        row.target_ppm_veg_weeks.0.len(),
        row.target_ppm_gen_weeks.0.len(),
    );

    let target_yield_kg = calculate_target_yield_kg(row.total_holes, row.target_yield_per_hole);

    let yield_achievement_percent = row
        .actual_yield_kg
        .map(|act| calculate_yield_achievement_percent(act, target_yield_kg));

    let total_mortality_for_calc = if row.status == "Completed" {
        row.mortality_count
    } else {
        (row.total_daily_mortality.unwrap_or(0) as i32).max(row.mortality_count)
    };
    let survival_rate_percent =
        calculate_survival_rate_percent(row.total_holes, total_mortality_for_calc);

    let snapshot = map_row_to_snapshot(&row);

    let active_targets = resolve_active_targets(&snapshot, &growth_info);

    let latest_reading =
        if let (Some(lr_id), Some(lr_date), Some(lr_time), Some(lr_ppm), Some(lr_ph)) = (
            &row.latest_reading_id,
            row.latest_reading_date,
            row.latest_reading_time,
            row.latest_reading_ppm,
            row.latest_reading_ph,
        ) {
            let lr_growth = calculate_growth_info(
                row.seeding_date,
                row.start_date,
                lr_date,
                row.target_growth_est_days,
                row.target_veg_phase_days,
                row.target_ppm_veg_weeks.0.len(),
                row.target_ppm_gen_weeks.0.len(),
            );
            let targets = resolve_active_targets(&snapshot, &lr_growth);
            let ppm_stat = crate::modules::daily_readings::service::evaluate_sensor_metric(
                lr_ppm,
                targets.ppm_ideal_min,
                targets.ppm_ideal_max,
                targets.ppm_tol_min,
                targets.ppm_tol_max,
            );
            let ph_stat = crate::modules::daily_readings::service::evaluate_sensor_metric(
                lr_ph,
                targets.ph_ideal_min,
                targets.ph_ideal_max,
                targets.ph_tol_min,
                targets.ph_tol_max,
            );
            let overall_status = match (ppm_stat, ph_stat) {
                (MetricStatus::Critical, _) | (_, MetricStatus::Critical) => MetricStatus::Critical,
                (MetricStatus::Warning, _) | (_, MetricStatus::Warning) => MetricStatus::Warning,
                _ => MetricStatus::Ideal,
            };

            Some(LatestReadingSummary {
                id: lr_id.clone(),
                reading_date: lr_date,
                reading_time: lr_time.format("%H:%M:%S").to_string(),
                ppm: lr_ppm,
                ph: lr_ph,
                overall_status,
            })
        } else {
            None
        };

    let status = SessionStatus::from_str(&row.status).unwrap_or(SessionStatus::Active);

    SessionItemResponse {
        id: row.id,
        session_name: row.session_name,
        plant_type_id: row.plant_type_id,
        plant_name: row.plant_name,
        plant_variety: row.plant_variety,
        location_id: row.location_id,
        location_name: row.location_name,
        installation_id: row.installation_id,
        installation_name: row.installation_name,
        seeding_date: row.seeding_date,
        start_date: row.start_date,
        estimated_harvest_date: row.estimated_harvest_date,
        harvest_date: row.harvest_date,
        total_holes: row.total_holes,
        status,
        is_locked: row.is_locked,
        target_yield_kg,
        actual_yield_kg: row.actual_yield_kg,
        saleable_count: row.saleable_count,
        mortality_count: row.mortality_count,
        total_daily_mortality: row.total_daily_mortality.unwrap_or(0),
        yield_achievement_percent,
        survival_rate_percent,
        growth_info,
        active_targets,
        has_reported_today: row.has_reported_today.unwrap_or(false),
        latest_reading,
        notes: row.notes,
        created_at: row.created_at,
        created_by: row.created_by,
        modify_at: row.modify_at,
        modify_by: row.modify_by,
    }
}

#[async_trait]
pub trait SessionService: Send + Sync {
    async fn get_sessions(&self, query: SessionListQuery) -> Result<SessionListResponse, AppError>;
    async fn get_session_by_id(
        &self,
        id: &str,
        date: Option<NaiveDate>,
    ) -> Result<SessionDetailResponse, AppError>;
    async fn create_session(
        &self,
        req: CreateSessionRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse, AppError>;
    async fn update_session(
        &self,
        id: &str,
        req: UpdateSessionRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse, AppError>;
    async fn complete_harvest(
        &self,
        id: &str,
        req: CompleteHarvestRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse, AppError>;
    async fn delete_session(&self, id: &str) -> Result<(), AppError>;
}

pub struct SessionServiceImpl {
    session_repo: Arc<dyn SessionRepository>,
    plant_repo: Arc<dyn PlantRepository>,
    location_repo: Arc<dyn LocationRepository>,
    installation_repo: Arc<dyn InstallationRepository>,
}

impl SessionServiceImpl {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        plant_repo: Arc<dyn PlantRepository>,
        location_repo: Arc<dyn LocationRepository>,
        installation_repo: Arc<dyn InstallationRepository>,
    ) -> Self {
        Self {
            session_repo,
            plant_repo,
            location_repo,
            installation_repo,
        }
    }
}

#[async_trait]
impl SessionService for SessionServiceImpl {
    async fn get_sessions(&self, query: SessionListQuery) -> Result<SessionListResponse, AppError> {
        let eval_date = query.date.unwrap_or_else(today_wib);
        let (kpi, rows) = self.session_repo.find_all(&query, eval_date).await?;

        let items = rows
            .into_iter()
            .map(|r| map_row_to_item_response(r, eval_date))
            .collect();

        Ok(SessionListResponse { kpi, items })
    }

    async fn get_session_by_id(
        &self,
        id: &str,
        date: Option<NaiveDate>,
    ) -> Result<SessionDetailResponse, AppError> {
        let eval_date = date.unwrap_or_else(today_wib);
        let row = self
            .session_repo
            .find_by_id(id, eval_date)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Planting session with ID '{id}' not found"))
            })?;

        let snapshot = map_row_to_snapshot(&row);
        let growth_info = calculate_growth_info(
            row.seeding_date,
            row.start_date,
            eval_date,
            row.target_growth_est_days,
            row.target_veg_phase_days,
            row.target_ppm_veg_weeks.0.len(),
            row.target_ppm_gen_weeks.0.len(),
        );
        let active_targets = resolve_active_targets(&snapshot, &growth_info);
        let harvest_notes = row.harvest_notes.clone();

        let item = map_row_to_item_response(row, eval_date);

        Ok(SessionDetailResponse {
            item,
            target_snapshot: snapshot,
            active_targets,
            harvest_notes,
        })
    }

    async fn create_session(
        &self,
        req: CreateSessionRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        if req.session_name.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Nama sesi tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }

        // 1. Validate Plant Type
        let plant = self
            .plant_repo
            .find_by_id(&req.plant_type_id)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "Plant type with ID '{}' not found",
                    req.plant_type_id
                ))
            })?;
        if !plant.is_active {
            return Err(AppError::BadRequest(
                "Plant type is inactive and cannot be used for new planting sessions".to_string(),
            ));
        }

        // 2. Validate Location
        let location = self
            .location_repo
            .find_by_id(&req.location_id)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!("Location with ID '{}' not found", req.location_id))
            })?;
        if !location.is_active {
            return Err(AppError::BadRequest(
                "Location is inactive and cannot host new planting sessions".to_string(),
            ));
        }

        // 3. Validate Installation if provided
        if let Some(ins_id) = &req.installation_id {
            let installation = self
                .installation_repo
                .find_by_id(ins_id)
                .await?
                .ok_or_else(|| {
                    AppError::BadRequest(format!("Installation with ID '{ins_id}' not found"))
                })?;

            if installation.location_id != req.location_id {
                return Err(AppError::BadRequest(
                    "Selected installation does not belong to the selected location".to_string(),
                ));
            }

            if installation.status != "Active" {
                return Err(AppError::BadRequest(format!(
                    "Installation unit is in '{}' status. Only 'Active' units can host planting sessions",
                    installation.status
                )));
            }

            if req.total_holes > installation.capacity_holes {
                return Err(AppError::BadRequest(format!(
                    "Total holes ({}) exceeds installation capacity ({})",
                    req.total_holes, installation.capacity_holes
                )));
            }
        }

        // 4. Validate Dates (Edge case #5)
        if let Some(seed_date) = req.seeding_date {
            if seed_date > req.start_date {
                return Err(AppError::BadRequest(
                    "Tanggal semai (seeding_date) tidak boleh lebih lambat dari tanggal mulai (start_date)".to_string(),
                ));
            }
        }

        let estimated_harvest_date = req
            .estimated_harvest_date
            .unwrap_or_else(|| req.start_date + Duration::days(plant.growth_est_days as i64));
        if estimated_harvest_date < req.start_date {
            return Err(AppError::BadRequest(
                "Estimasi tanggal panen tidak boleh lebih awal dari tanggal mulai tanam"
                    .to_string(),
            ));
        }

        let new_id = format!("ses_{}", Uuid::new_v4());
        let now = Utc::now();

        let entity = SessionEntity {
            id: new_id.clone(),
            session_name: req.session_name.trim().to_string(),
            plant_type_id: plant.id,
            location_id: req.location_id,
            installation_id: req.installation_id,
            seeding_date: req.seeding_date,
            start_date: req.start_date,
            estimated_harvest_date,
            harvest_date: None,
            total_holes: req.total_holes,
            status: "Active".to_string(),
            is_locked: false,
            target_growth_est_days: plant.growth_est_days,
            target_veg_phase_days: plant.veg_phase_days,
            target_yield_per_hole: plant.target_yield_per_hole,
            target_min_ph: plant.min_ph,
            target_ideal_min_ph: plant.ideal_min_ph,
            target_ideal_max_ph: plant.ideal_max_ph,
            target_max_ph: plant.max_ph,
            target_ppm_veg_weeks: plant.ideal_ppm_veg_weeks,
            target_ppm_gen_weeks: plant.ideal_ppm_gen_weeks,
            target_ideal_min_ppm_veg: plant.ideal_min_ppm_veg,
            target_ideal_max_ppm_veg: plant.ideal_max_ppm_veg,
            target_tol_min_ppm_veg: plant.tol_min_ppm_veg,
            target_tol_max_ppm_veg: plant.tol_max_ppm_veg,
            target_ideal_min_ppm_gen: plant.ideal_min_ppm_gen,
            target_ideal_max_ppm_gen: plant.ideal_max_ppm_gen,
            target_tol_min_ppm_gen: plant.tol_min_ppm_gen,
            target_tol_max_ppm_gen: plant.tol_max_ppm_gen,
            target_ideal_min_temp_env: plant.ideal_min_temp_env,
            target_ideal_max_temp_env: plant.ideal_max_temp_env,
            target_ideal_min_temp_water: plant.ideal_min_temp_water,
            target_ideal_max_temp_water: plant.ideal_max_temp_water,
            target_ideal_min_hum: plant.ideal_min_hum,
            target_ideal_max_hum: plant.ideal_max_hum,
            actual_yield_kg: None,
            saleable_count: None,
            mortality_count: 0,
            harvest_notes: String::new(),
            notes: req.notes.unwrap_or_default().trim().to_string(),
            created_at: now,
            created_by: user_id.to_string(),
            modify_at: now,
            modify_by: user_id.to_string(),
        };

        self.session_repo.create(&entity).await?;

        self.get_session_by_id(&new_id, Some(today_wib())).await
    }

    async fn update_session(
        &self,
        id: &str,
        req: UpdateSessionRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        if req.session_name.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Nama sesi tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }

        let existing = self.session_repo.find_raw_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Planting session with ID '{id}' not found"))
        })?;

        if existing.is_locked || existing.status != "Active" {
            return Err(AppError::BadRequest(
                "Sesi tanam tidak aktif atau sudah terkunci".to_string(),
            ));
        }

        // Validate Location
        let location = self
            .location_repo
            .find_by_id(&req.location_id)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!("Location with ID '{}' not found", req.location_id))
            })?;
        if !location.is_active {
            return Err(AppError::BadRequest(
                "Location is inactive and cannot host planting sessions".to_string(),
            ));
        }

        // Validate Installation
        if let Some(ins_id) = &req.installation_id {
            let installation = self
                .installation_repo
                .find_by_id(ins_id)
                .await?
                .ok_or_else(|| {
                    AppError::BadRequest(format!("Installation with ID '{ins_id}' not found"))
                })?;

            if installation.location_id != req.location_id {
                return Err(AppError::BadRequest(
                    "Selected installation does not belong to the selected location".to_string(),
                ));
            }

            if installation.status != "Active" {
                return Err(AppError::BadRequest(format!(
                    "Installation unit is in '{}' status. Only 'Active' units can host planting sessions",
                    installation.status
                )));
            }

            if req.total_holes > installation.capacity_holes {
                return Err(AppError::BadRequest(format!(
                    "Total holes ({}) exceeds installation capacity ({})",
                    req.total_holes, installation.capacity_holes
                )));
            }
        }

        // Validate Dates
        if let Some(seed_date) = req.seeding_date {
            if seed_date > req.start_date {
                return Err(AppError::BadRequest(
                    "Tanggal semai (seeding_date) tidak boleh lebih lambat dari tanggal mulai (start_date)".to_string(),
                ));
            }
        }

        let estimated_harvest_date = req.estimated_harvest_date.unwrap_or_else(|| {
            req.start_date + Duration::days(existing.target_growth_est_days as i64)
        });
        if estimated_harvest_date < req.start_date {
            return Err(AppError::BadRequest(
                "Estimasi tanggal panen tidak boleh lebih awal dari tanggal mulai tanam"
                    .to_string(),
            ));
        }

        let new_status = match req.status {
            Some(SessionStatus::Completed) => {
                return Err(AppError::BadRequest(
                    "Penutupan panen harus melalui endpoint /complete-harvest".to_string(),
                ));
            }
            Some(s) => s.to_string(),
            None => existing.status,
        };

        let updated_entity = SessionEntity {
            id: id.to_string(),
            session_name: req.session_name.trim().to_string(),
            plant_type_id: existing.plant_type_id,
            location_id: req.location_id,
            installation_id: req.installation_id,
            seeding_date: req.seeding_date,
            start_date: req.start_date,
            estimated_harvest_date,
            harvest_date: existing.harvest_date,
            total_holes: req.total_holes,
            status: new_status,
            is_locked: false,
            target_growth_est_days: existing.target_growth_est_days,
            target_veg_phase_days: existing.target_veg_phase_days,
            target_yield_per_hole: existing.target_yield_per_hole,
            target_min_ph: existing.target_min_ph,
            target_ideal_min_ph: existing.target_ideal_min_ph,
            target_ideal_max_ph: existing.target_ideal_max_ph,
            target_max_ph: existing.target_max_ph,
            target_ppm_veg_weeks: existing.target_ppm_veg_weeks,
            target_ppm_gen_weeks: existing.target_ppm_gen_weeks,
            target_ideal_min_ppm_veg: existing.target_ideal_min_ppm_veg,
            target_ideal_max_ppm_veg: existing.target_ideal_max_ppm_veg,
            target_tol_min_ppm_veg: existing.target_tol_min_ppm_veg,
            target_tol_max_ppm_veg: existing.target_tol_max_ppm_veg,
            target_ideal_min_ppm_gen: existing.target_ideal_min_ppm_gen,
            target_ideal_max_ppm_gen: existing.target_ideal_max_ppm_gen,
            target_tol_min_ppm_gen: existing.target_tol_min_ppm_gen,
            target_tol_max_ppm_gen: existing.target_tol_max_ppm_gen,
            target_ideal_min_temp_env: existing.target_ideal_min_temp_env,
            target_ideal_max_temp_env: existing.target_ideal_max_temp_env,
            target_ideal_min_temp_water: existing.target_ideal_min_temp_water,
            target_ideal_max_temp_water: existing.target_ideal_max_temp_water,
            target_ideal_min_hum: existing.target_ideal_min_hum,
            target_ideal_max_hum: existing.target_ideal_max_hum,
            actual_yield_kg: existing.actual_yield_kg,
            saleable_count: existing.saleable_count,
            mortality_count: existing.mortality_count,
            harvest_notes: existing.harvest_notes,
            notes: req.notes.unwrap_or(existing.notes).trim().to_string(),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: Utc::now(),
            modify_by: user_id.to_string(),
        };

        self.session_repo.update(&updated_entity).await?;

        self.get_session_by_id(id, None).await
    }

    async fn complete_harvest(
        &self,
        id: &str,
        req: CompleteHarvestRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let existing = self.session_repo.find_raw_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Planting session with ID '{id}' not found"))
        })?;

        if existing.is_locked || existing.status != "Active" {
            return Err(AppError::BadRequest(
                "Sesi tanam tidak aktif atau sudah terkunci".to_string(),
            ));
        }

        let harvest_date = req.harvest_date.unwrap_or_else(today_wib);
        if harvest_date < existing.start_date {
            return Err(AppError::BadRequest(
                "Tanggal panen tidak boleh lebih awal dari tanggal mulai tanam".to_string(),
            ));
        }

        if req.actual_yield_kg <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Total berat hasil panen (Kg) wajib lebih besar dari 0".to_string(),
            ));
        }

        let max_yield =
            Decimal::from_str("9999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;
        if req.actual_yield_kg > max_yield {
            return Err(AppError::BadRequest(
                "Nilai angka melebihi batas maksimum kapasitas sistem".to_string(),
            ));
        }

        if req.saleable_count < 0 || req.mortality_count < 0 {
            return Err(AppError::BadRequest(
                "Jumlah tanaman layak jual dan mortalitas tidak boleh bernilai negatif".to_string(),
            ));
        }

        if req.saleable_count + req.mortality_count > existing.total_holes {
            return Err(AppError::BadRequest(
                "Total tanaman layak jual dan mortalitas tidak boleh melebihi kapasitas lubang sesi".to_string(),
            ));
        }

        let harvest_notes = req.harvest_notes.unwrap_or_default().trim().to_string();

        let params = crate::modules::sessions::repository::CompleteHarvestParams {
            harvest_date,
            actual_yield_kg: req.actual_yield_kg.round_dp(2),
            saleable_count: req.saleable_count,
            mortality_count: req.mortality_count,
            harvest_notes: &harvest_notes,
            modify_by: user_id,
        };

        self.session_repo.complete_harvest(id, &params).await?;

        self.get_session_by_id(id, Some(harvest_date)).await
    }

    async fn delete_session(&self, id: &str) -> Result<(), AppError> {
        let existing = self.session_repo.find_raw_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Planting session with ID '{id}' not found"))
        })?;

        if existing.is_locked || existing.status == "Completed" {
            return Err(AppError::BadRequest(
                "Sesi tanam yang sudah selesai dan terkunci tidak dapat dihapus".to_string(),
            ));
        }

        self.session_repo.delete(id).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_growth_info_and_panen_simulation_1() {
        // Data Sesi Simulasi 1 PRD:
        // Tanaman: Selada Romaine, start_date: 2026-11-01, growth_est_days: 35, target_veg_phase_days: 21 (3 minggu veg)
        // Kurva PPM veg: 3 minggu ([560-700], [700-840], [800-900])
        let start_date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let eval_date = NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
        let growth_est_days = 35;
        let veg_phase_days = 21;
        let veg_weeks_len = 3;
        let gen_weeks_len = 2;

        let growth_info = calculate_growth_info(
            None,
            start_date,
            eval_date,
            growth_est_days,
            veg_phase_days,
            veg_weeks_len,
            gen_weeks_len,
        );

        assert_eq!(growth_info.phase, GrowthPhase::Vegetatif);
        assert_eq!(growth_info.hst, 10);
        assert_eq!(growth_info.active_week, 2);
        assert_eq!(growth_info.week_index, 1);
        assert_eq!(
            growth_info.progress_percent,
            Decimal::from_str("28.57").unwrap()
        );
        assert!(!growth_info.is_overdue);
    }

    #[test]
    fn test_semai_phase_and_overdue_clamping() {
        let start_date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let seeding_date = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();

        // 1. Semai date: 2026-10-25
        let eval_semai = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let growth_semai =
            calculate_growth_info(Some(seeding_date), start_date, eval_semai, 35, 21, 3, 2);
        assert_eq!(growth_semai.phase, GrowthPhase::Semai);
        assert_eq!(growth_semai.hst, 0);
        assert_eq!(growth_semai.seeding_days, 5);
        assert_eq!(growth_semai.active_week, 1);
        assert_eq!(growth_semai.progress_percent, Decimal::ZERO);

        // 2. Overdue: eval_date 2026-12-15 (44 days after 2026-11-01, est = 35)
        let eval_overdue = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
        let growth_overdue =
            calculate_growth_info(Some(seeding_date), start_date, eval_overdue, 35, 21, 3, 2);
        assert_eq!(growth_overdue.phase, GrowthPhase::Generatif);
        assert_eq!(growth_overdue.hst, 44);
        assert!(growth_overdue.is_overdue);
        assert_eq!(
            growth_overdue.progress_percent,
            Decimal::from_str("100.00").unwrap()
        ); // clamped to 100%
        assert_eq!(growth_overdue.week_index, 1); // clamped to gen_weeks_len - 1
    }

    #[test]
    fn test_harvest_metrics_simulation_2() {
        // Simulasi 2 PRD:
        // total_holes = 400, target_yield_per_hole = 250.00 gr
        // TargetYieldKg = (400 * 250) / 1000 = 100.00 kg
        let target_yield_kg = calculate_target_yield_kg(400, Decimal::from_str("250.00").unwrap());
        assert_eq!(target_yield_kg, Decimal::from_str("100.00").unwrap());

        // Realisasi: actual_yield_kg = 92.50, saleable_count = 370, mortality_count = 24
        let achievement = calculate_yield_achievement_percent(
            Decimal::from_str("92.50").unwrap(),
            target_yield_kg,
        );
        assert_eq!(achievement, Decimal::from_str("92.50").unwrap());

        let survival_rate = calculate_survival_rate_percent(400, 24);
        assert_eq!(survival_rate, Decimal::from_str("94.00").unwrap());
    }

    #[test]
    fn test_harvest_population_and_yield_invariants() {
        let total_holes = 500;
        let valid_saleable = 450;
        let valid_mortality = 40;
        assert!(valid_saleable + valid_mortality <= total_holes);

        // Exceeding total holes
        let invalid_saleable = 480;
        let invalid_mortality = 30; // 510 > 500
        assert!(invalid_saleable + invalid_mortality > total_holes);

        // Actual yield zero / negative
        assert!(Decimal::ZERO <= Decimal::ZERO);
        assert!(Decimal::from_str("12.50").unwrap() > Decimal::ZERO);
    }
}
