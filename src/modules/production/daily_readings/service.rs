use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::daily_readings::dto::{
    ActiveMetricTargets, CreateDailyReadingRequest, DailyReadingDiagnostics, DailyReadingEntity,
    DailyReadingItem, DailyReadingListQuery, MetricStatus, SessionDailyReadingsHeader,
    SessionDailyReadingsResponse, UnreportedTodayQuery, UnreportedTodayResponse,
};
use crate::modules::daily_readings::repository::DailyReadingRepository;
use crate::modules::masters::plants::dto::PpmWeekRange;
use crate::modules::sessions::dto::{
    GrowthInfo, GrowthPhase, SessionStatus, SessionTargetSnapshot,
};
use crate::modules::sessions::repository::SessionRepository;
use crate::modules::sessions::service::{
    calculate_growth_info, map_row_to_item_response, map_row_to_snapshot, today_wib,
};

pub fn evaluate_sensor_metric(
    value: Decimal,
    ideal_min: Decimal,
    ideal_max: Decimal,
    tol_min: Decimal,
    tol_max: Decimal,
) -> MetricStatus {
    if value >= ideal_min && value <= ideal_max {
        MetricStatus::Ideal
    } else if (value >= tol_min && value < ideal_min) || (value > ideal_max && value <= tol_max) {
        MetricStatus::Warning
    } else {
        MetricStatus::Critical
    }
}

pub fn resolve_active_targets(
    snapshot: &SessionTargetSnapshot,
    growth_info: &GrowthInfo,
) -> ActiveMetricTargets {
    let (ppm_ideal_min, ppm_ideal_max, ppm_tol_min, ppm_tol_max) = match growth_info.phase {
        GrowthPhase::Semai | GrowthPhase::Vegetatif => {
            let range = snapshot
                .target_ppm_veg_weeks
                .get(growth_info.week_index)
                .cloned()
                .unwrap_or(PpmWeekRange {
                    min: snapshot.target_ideal_min_ppm_veg,
                    max: snapshot.target_ideal_max_ppm_veg,
                });
            let w_min = range.min;
            let w_max = range.max;
            let delta_min =
                (snapshot.target_ideal_min_ppm_veg - snapshot.target_tol_min_ppm_veg).max(50);
            let delta_max =
                (snapshot.target_tol_max_ppm_veg - snapshot.target_ideal_max_ppm_veg).max(50);
            let t_min = (w_min - delta_min).max(0);
            let t_max = w_max + delta_max;
            (
                Decimal::from(w_min),
                Decimal::from(w_max),
                Decimal::from(t_min),
                Decimal::from(t_max),
            )
        }
        GrowthPhase::Generatif => {
            let range = snapshot
                .target_ppm_gen_weeks
                .get(growth_info.week_index)
                .cloned()
                .or_else(|| snapshot.target_ppm_veg_weeks.last().cloned())
                .unwrap_or(PpmWeekRange {
                    min: snapshot.target_ideal_min_ppm_gen,
                    max: snapshot.target_ideal_max_ppm_gen,
                });
            let w_min = range.min;
            let w_max = range.max;
            let delta_min =
                (snapshot.target_ideal_min_ppm_gen - snapshot.target_tol_min_ppm_gen).max(50);
            let delta_max =
                (snapshot.target_tol_max_ppm_gen - snapshot.target_ideal_max_ppm_gen).max(50);
            let t_min = (w_min - delta_min).max(0);
            let t_max = w_max + delta_max;
            (
                Decimal::from(w_min),
                Decimal::from(w_max),
                Decimal::from(t_min),
                Decimal::from(t_max),
            )
        }
    };

    let ph_ideal_min = snapshot.target_ideal_min_ph;
    let ph_ideal_max = snapshot.target_ideal_max_ph;
    let ph_tol_min = snapshot.target_min_ph;
    let ph_tol_max = snapshot.target_max_ph;

    let temp_env_ideal_min = snapshot.target_ideal_min_temp_env;
    let temp_env_ideal_max = snapshot.target_ideal_max_temp_env;

    let temp_water_ideal_min = snapshot.target_ideal_min_temp_water;
    let temp_water_ideal_max = snapshot.target_ideal_max_temp_water;

    let hum_ideal_min = snapshot.target_ideal_min_hum;
    let hum_ideal_max = snapshot.target_ideal_max_hum;

    ActiveMetricTargets {
        ppm_ideal_min,
        ppm_ideal_max,
        ppm_tol_min,
        ppm_tol_max,
        ph_ideal_min,
        ph_ideal_max,
        ph_tol_min,
        ph_tol_max,
        temp_env_ideal_min,
        temp_env_ideal_max,
        temp_water_ideal_min,
        temp_water_ideal_max,
        hum_ideal_min,
        hum_ideal_max,
    }
}

pub fn evaluate_daily_reading_diagnostics(
    reading: &DailyReadingEntity,
    snapshot: &SessionTargetSnapshot,
    growth_info: &GrowthInfo,
) -> DailyReadingDiagnostics {
    let active_targets = resolve_active_targets(snapshot, growth_info);

    let ppm_status = evaluate_sensor_metric(
        reading.ppm,
        active_targets.ppm_ideal_min,
        active_targets.ppm_ideal_max,
        active_targets.ppm_tol_min,
        active_targets.ppm_tol_max,
    );

    let ph_status = evaluate_sensor_metric(
        reading.ph,
        active_targets.ph_ideal_min,
        active_targets.ph_ideal_max,
        active_targets.ph_tol_min,
        active_targets.ph_tol_max,
    );

    let temp_env_status = reading.temp_env.map(|v| {
        let two = Decimal::from(2);
        let zero = Decimal::ZERO;
        evaluate_sensor_metric(
            v,
            active_targets.temp_env_ideal_min,
            active_targets.temp_env_ideal_max,
            (active_targets.temp_env_ideal_min - two).max(zero),
            active_targets.temp_env_ideal_max + two,
        )
    });

    let temp_water_status = reading.temp_water.map(|v| {
        let two = Decimal::from(2);
        let zero = Decimal::ZERO;
        evaluate_sensor_metric(
            v,
            active_targets.temp_water_ideal_min,
            active_targets.temp_water_ideal_max,
            (active_targets.temp_water_ideal_min - two).max(zero),
            active_targets.temp_water_ideal_max + two,
        )
    });

    let humidity_status = reading.humidity.map(|v| {
        let ten = Decimal::from(10);
        let zero = Decimal::ZERO;
        let hundred = Decimal::ONE_HUNDRED;
        evaluate_sensor_metric(
            v,
            active_targets.hum_ideal_min,
            active_targets.hum_ideal_max,
            (active_targets.hum_ideal_min - ten).max(zero),
            (active_targets.hum_ideal_max + ten).min(hundred),
        )
    });

    let mut overall = MetricStatus::Ideal;
    let mut check_status = |s: MetricStatus| {
        if s == MetricStatus::Critical {
            overall = MetricStatus::Critical;
        } else if s == MetricStatus::Warning && overall != MetricStatus::Critical {
            overall = MetricStatus::Warning;
        }
    };

    check_status(ppm_status);
    check_status(ph_status);
    if let Some(s) = temp_env_status {
        check_status(s);
    }
    if let Some(s) = temp_water_status {
        check_status(s);
    }
    if let Some(s) = humidity_status {
        check_status(s);
    }

    DailyReadingDiagnostics {
        ppm_status,
        ph_status,
        temp_env_status,
        temp_water_status,
        humidity_status,
        overall_status: overall,
        active_targets,
    }
}

pub fn map_entity_to_reading_item(
    entity: DailyReadingEntity,
    snapshot: &SessionTargetSnapshot,
    seeding_date: Option<NaiveDate>,
    start_date: NaiveDate,
) -> DailyReadingItem {
    let growth_info = calculate_growth_info(
        seeding_date,
        start_date,
        entity.reading_date,
        snapshot.target_growth_est_days,
        snapshot.target_veg_phase_days,
        snapshot.target_ppm_veg_weeks.len(),
        snapshot.target_ppm_gen_weeks.len(),
    );

    let diagnostics = evaluate_daily_reading_diagnostics(&entity, snapshot, &growth_info);

    DailyReadingItem {
        id: entity.id,
        session_id: entity.session_id,
        reading_date: entity.reading_date,
        reading_time: entity.reading_time.format("%H:%M:%S").to_string(),
        ppm: entity.ppm,
        ph: entity.ph,
        temp_env: entity.temp_env,
        temp_water: entity.temp_water,
        humidity: entity.humidity,
        mortality_count: entity.mortality_count,
        notes: entity.notes,
        growth_info,
        diagnostics,
        created_at: entity.created_at,
        created_by: entity.created_by,
    }
}

#[async_trait]
pub trait DailyReadingService: Send + Sync {
    async fn get_daily_readings_by_session(
        &self,
        session_id: &str,
        query: DailyReadingListQuery,
    ) -> Result<SessionDailyReadingsResponse, AppError>;

    async fn create_daily_reading(
        &self,
        session_id: &str,
        req: CreateDailyReadingRequest,
        user_id: &str,
    ) -> Result<DailyReadingItem, AppError>;

    async fn delete_daily_reading(&self, id: &str) -> Result<(), AppError>;

    async fn get_unreported_today(
        &self,
        query: UnreportedTodayQuery,
    ) -> Result<UnreportedTodayResponse, AppError>;
}

pub struct DailyReadingServiceImpl {
    daily_reading_repo: Arc<dyn DailyReadingRepository>,
    session_repo: Arc<dyn SessionRepository>,
}

impl DailyReadingServiceImpl {
    pub fn new(
        daily_reading_repo: Arc<dyn DailyReadingRepository>,
        session_repo: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            daily_reading_repo,
            session_repo,
        }
    }
}

#[async_trait]
impl DailyReadingService for DailyReadingServiceImpl {
    async fn get_daily_readings_by_session(
        &self,
        session_id: &str,
        query: DailyReadingListQuery,
    ) -> Result<SessionDailyReadingsResponse, AppError> {
        let eval_date = today_wib();
        let session_row = self
            .session_repo
            .find_by_id(session_id, eval_date)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Planting session with ID '{session_id}' not found"))
            })?;

        let snapshot = map_row_to_snapshot(&session_row);
        let header_growth = calculate_growth_info(
            session_row.seeding_date,
            session_row.start_date,
            eval_date,
            session_row.target_growth_est_days,
            session_row.target_veg_phase_days,
            session_row.target_ppm_veg_weeks.0.len(),
            session_row.target_ppm_gen_weeks.0.len(),
        );
        let active_targets = resolve_active_targets(&snapshot, &header_growth);

        let (total_daily_mortality, total_readings, latest_entity, reading_entities) = self
            .daily_reading_repo
            .find_by_session_id(session_id, &query)
            .await?;

        let latest_reading = latest_entity.map(|ent| {
            map_entity_to_reading_item(
                ent,
                &snapshot,
                session_row.seeding_date,
                session_row.start_date,
            )
        });

        let mut readings: Vec<DailyReadingItem> = reading_entities
            .into_iter()
            .map(|ent| {
                map_entity_to_reading_item(
                    ent,
                    &snapshot,
                    session_row.seeding_date,
                    session_row.start_date,
                )
            })
            .collect();

        if let Some(target_status) = query.overall_status {
            readings.retain(|r| r.diagnostics.overall_status == target_status);
        }

        let session_status =
            SessionStatus::from_str(&session_row.status).unwrap_or(SessionStatus::Active);

        let target_yield_kg = crate::modules::sessions::service::calculate_target_yield_kg(
            session_row.total_holes,
            session_row.target_yield_per_hole,
        );

        let session = SessionDailyReadingsHeader {
            id: session_row.id,
            session_name: session_row.session_name,
            plant_name: session_row.plant_name,
            plant_variety: session_row.plant_variety,
            location_name: session_row.location_name,
            installation_name: session_row.installation_name,
            start_date: session_row.start_date,
            estimated_harvest_date: session_row.estimated_harvest_date,
            total_holes: session_row.total_holes,
            target_yield_kg,
            status: session_status,
            is_locked: session_row.is_locked,
            growth_info: header_growth,
            active_targets,
        };

        Ok(SessionDailyReadingsResponse {
            session,
            total_daily_mortality,
            total_readings,
            latest_reading,
            readings,
        })
    }

    async fn create_daily_reading(
        &self,
        session_id: &str,
        req: CreateDailyReadingRequest,
        user_id: &str,
    ) -> Result<DailyReadingItem, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let ppm_max = Decimal::from(50_000);
        if req.ppm < Decimal::ZERO || req.ppm > ppm_max {
            return Err(AppError::BadRequest(
                "PPM harus berada dalam rentang 0.00 hingga 50,000.00".to_string(),
            ));
        }

        let ph_min = Decimal::ZERO;
        let ph_max = Decimal::from(14);
        if req.ph < ph_min || req.ph > ph_max {
            return Err(AppError::BadRequest(
                "pH harus berada dalam rentang 0.00 hingga 14.00".to_string(),
            ));
        }

        let temp_min = Decimal::from(-50);
        let temp_max = Decimal::from(100);
        if let Some(t) = req.temp_env {
            if t < temp_min || t > temp_max {
                return Err(AppError::BadRequest(
                    "Suhu lingkungan harus berada dalam rentang -50.00 hingga 100.00 °C"
                        .to_string(),
                ));
            }
        }
        if let Some(t) = req.temp_water {
            if t < temp_min || t > temp_max {
                return Err(AppError::BadRequest(
                    "Suhu air nutrisi harus berada dalam rentang -50.00 hingga 100.00 °C"
                        .to_string(),
                ));
            }
        }

        if let Some(h) = req.humidity {
            let hum_max = Decimal::ONE_HUNDRED;
            if h < Decimal::ZERO || h > hum_max {
                return Err(AppError::BadRequest(
                    "Kelembapan harus berada dalam rentang 0.00 hingga 100.00".to_string(),
                ));
            }
        }

        let eval_date = today_wib();
        let session_row = self
            .session_repo
            .find_by_id(session_id, eval_date)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Planting session with ID '{session_id}' not found"))
            })?;

        if session_row.is_locked || session_row.status != "Active" {
            return Err(AppError::BadRequest(
                "Sesi tanam tidak aktif atau sudah terkunci".to_string(),
            ));
        }

        let reading_date = req.reading_date.unwrap_or_else(today_wib);
        let reading_time = if let Some(time_str) = req.reading_time {
            NaiveTime::parse_from_str(&time_str, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(&time_str, "%H:%M"))
                .map_err(|_| {
                    AppError::BadRequest(
                        "Format jam tidak valid. Gunakan format HH:MM:SS".to_string(),
                    )
                })?
        } else {
            let now_wib = Utc::now() + chrono::Duration::hours(7);
            now_wib.time()
        };

        let new_id = format!("drd_{}", Uuid::new_v4());
        let now = Utc::now();

        let entity = DailyReadingEntity {
            id: new_id,
            session_id: session_id.to_string(),
            reading_date,
            reading_time,
            ppm: req.ppm.round_dp(2),
            ph: req.ph.round_dp(2),
            temp_env: req.temp_env.map(|v| v.round_dp(2)),
            temp_water: req.temp_water.map(|v| v.round_dp(2)),
            humidity: req.humidity.map(|v| v.round_dp(2)),
            mortality_count: req.mortality_count.unwrap_or(0),
            notes: req.notes.unwrap_or_default().trim().to_string(),
            created_at: now,
            created_by: user_id.to_string(),
        };

        self.daily_reading_repo.create(&entity).await?;

        let snapshot = map_row_to_snapshot(&session_row);
        let item = map_entity_to_reading_item(
            entity,
            &snapshot,
            session_row.seeding_date,
            session_row.start_date,
        );

        Ok(item)
    }

    async fn delete_daily_reading(&self, id: &str) -> Result<(), AppError> {
        self.daily_reading_repo.delete(id).await
    }

    async fn get_unreported_today(
        &self,
        query: UnreportedTodayQuery,
    ) -> Result<UnreportedTodayResponse, AppError> {
        let eval_date = query.date.unwrap_or_else(today_wib);
        let (total_active_sessions, reported_count, unreported_count, rows) = self
            .daily_reading_repo
            .find_unreported_active_sessions(eval_date, query.location_id.as_deref())
            .await?;

        let unreported_sessions = rows
            .into_iter()
            .map(|r| map_row_to_item_response(r, eval_date))
            .collect();

        Ok(UnreportedTodayResponse {
            evaluation_date: eval_date,
            total_active_sessions,
            reported_count,
            unreported_count,
            unreported_sessions,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_sensor_diagnostics_and_ppm_week_simulation_1() {
        // Simulasi 1 PRD:
        // Selada Romaine, HST = 10 -> Vegetatif Minggu ke-2
        // Minggu 2: Ideal 700 - 840, Toleransi 600 - 940 (delta_min = 100, delta_max = 100)
        // Standar pH: Ideal 5.50 - 6.50, Toleransi 5.00 - 7.00
        let ppm_ideal_min = Decimal::from_str("700.0").unwrap();
        let ppm_ideal_max = Decimal::from_str("840.0").unwrap();
        let ppm_tol_min = Decimal::from_str("600.0").unwrap();
        let ppm_tol_max = Decimal::from_str("940.0").unwrap();

        let ph_ideal_min = Decimal::from_str("5.50").unwrap();
        let ph_ideal_max = Decimal::from_str("6.50").unwrap();
        let ph_tol_min = Decimal::from_str("5.00").unwrap();
        let ph_tol_max = Decimal::from_str("7.00").unwrap();

        // Uji PPM:
        // ppm = 750.00 -> IDEAL
        assert_eq!(
            evaluate_sensor_metric(
                Decimal::from_str("750.00").unwrap(),
                ppm_ideal_min,
                ppm_ideal_max,
                ppm_tol_min,
                ppm_tol_max
            ),
            MetricStatus::Ideal
        );

        // ppm = 650.00 -> WARNING
        assert_eq!(
            evaluate_sensor_metric(
                Decimal::from_str("650.00").unwrap(),
                ppm_ideal_min,
                ppm_ideal_max,
                ppm_tol_min,
                ppm_tol_max
            ),
            MetricStatus::Warning
        );

        // ppm = 980.00 -> CRITICAL
        assert_eq!(
            evaluate_sensor_metric(
                Decimal::from_str("980.00").unwrap(),
                ppm_ideal_min,
                ppm_ideal_max,
                ppm_tol_min,
                ppm_tol_max
            ),
            MetricStatus::Critical
        );

        // Uji pH:
        // ph = 6.20 -> IDEAL
        assert_eq!(
            evaluate_sensor_metric(
                Decimal::from_str("6.20").unwrap(),
                ph_ideal_min,
                ph_ideal_max,
                ph_tol_min,
                ph_tol_max
            ),
            MetricStatus::Ideal
        );

        // ph = 4.80 -> CRITICAL
        assert_eq!(
            evaluate_sensor_metric(
                Decimal::from_str("4.80").unwrap(),
                ph_ideal_min,
                ph_ideal_max,
                ph_tol_min,
                ph_tol_max
            ),
            MetricStatus::Critical
        );
    }
}
