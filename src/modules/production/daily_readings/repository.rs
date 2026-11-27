use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::daily_readings::dto::{DailyReadingEntity, DailyReadingListQuery};
use crate::modules::sessions::dto::SessionRowWithRelations;

#[async_trait]
pub trait DailyReadingRepository: Send + Sync {
    async fn find_by_session_id(
        &self,
        session_id: &str,
        query: &DailyReadingListQuery,
    ) -> Result<
        (
            i64,
            i64,
            Option<DailyReadingEntity>,
            Vec<DailyReadingEntity>,
        ),
        AppError,
    >;

    async fn create(&self, reading: &DailyReadingEntity) -> Result<(), AppError>;

    async fn find_by_id(&self, id: &str) -> Result<Option<DailyReadingEntity>, AppError>;

    async fn delete(&self, id: &str) -> Result<(), AppError>;

    async fn find_unreported_active_sessions(
        &self,
        eval_date: NaiveDate,
        location_id: Option<&str>,
    ) -> Result<(i64, i64, i64, Vec<SessionRowWithRelations>), AppError>;
}

pub struct PgDailyReadingRepository {
    pool: PgPool,
}

impl PgDailyReadingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DailyReadingRepository for PgDailyReadingRepository {
    async fn find_by_session_id(
        &self,
        session_id: &str,
        query: &DailyReadingListQuery,
    ) -> Result<
        (
            i64,
            i64,
            Option<DailyReadingEntity>,
            Vec<DailyReadingEntity>,
        ),
        AppError,
    > {
        // 1. Total mortality sum for the session
        let (total_daily_mortality,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(mortality_count), 0)::BIGINT FROM daily_readings WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        // 2. Latest reading for the session (regardless of query pagination)
        let latest_reading: Option<DailyReadingEntity> = sqlx::query_as(
            r#"
            SELECT * FROM daily_readings
            WHERE session_id = $1
            ORDER BY reading_date DESC, reading_time DESC, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        // 3. Count total matching readings
        let mut count_sql =
            "SELECT COUNT(*)::BIGINT FROM daily_readings WHERE session_id = $1".to_string();
        let mut param_idx = 2;
        if query.start_date.is_some() {
            count_sql.push_str(&format!(" AND reading_date >= ${param_idx}"));
            param_idx += 1;
        }
        if query.end_date.is_some() {
            count_sql.push_str(&format!(" AND reading_date <= ${param_idx}"));
        }

        let mut count_q = sqlx::query_as::<_, (i64,)>(&count_sql).bind(session_id);
        if let Some(start) = query.start_date {
            count_q = count_q.bind(start);
        }
        if let Some(end) = query.end_date {
            count_q = count_q.bind(end);
        }
        let (total_readings,) = count_q.fetch_one(&self.pool).await?;

        // 4. Fetch list
        let sort_order = match query.sort_order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC",
        };

        let mut list_sql = "SELECT * FROM daily_readings WHERE session_id = $1".to_string();
        let mut list_param_idx = 2;
        if query.start_date.is_some() {
            list_sql.push_str(&format!(" AND reading_date >= ${list_param_idx}"));
            list_param_idx += 1;
        }
        if query.end_date.is_some() {
            list_sql.push_str(&format!(" AND reading_date <= ${list_param_idx}"));
            list_param_idx += 1;
        }

        list_sql.push_str(&format!(
            " ORDER BY reading_date {sort_order}, reading_time {sort_order}, created_at {sort_order}"
        ));

        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);
        list_sql.push_str(&format!(
            " LIMIT ${list_param_idx} OFFSET ${}",
            list_param_idx + 1
        ));

        let mut list_q = sqlx::query_as::<_, DailyReadingEntity>(&list_sql).bind(session_id);
        if let Some(start) = query.start_date {
            list_q = list_q.bind(start);
        }
        if let Some(end) = query.end_date {
            list_q = list_q.bind(end);
        }
        list_q = list_q.bind(limit).bind(offset);

        let readings = list_q.fetch_all(&self.pool).await?;

        Ok((
            total_daily_mortality,
            total_readings,
            latest_reading,
            readings,
        ))
    }

    async fn create(&self, reading: &DailyReadingEntity) -> Result<(), AppError> {
        let sql = r#"
            INSERT INTO daily_readings (
                id, session_id, reading_date, reading_time,
                ppm, ph, temp_env, temp_water, humidity,
                mortality_count, notes, created_at, created_by
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8, $9,
                $10, $11, $12, $13
            )
        "#;

        sqlx::query(sql)
            .bind(&reading.id)
            .bind(&reading.session_id)
            .bind(reading.reading_date)
            .bind(reading.reading_time)
            .bind(reading.ppm)
            .bind(reading.ph)
            .bind(reading.temp_env)
            .bind(reading.temp_water)
            .bind(reading.humidity)
            .bind(reading.mortality_count)
            .bind(&reading.notes)
            .bind(reading.created_at)
            .bind(&reading.created_by)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<DailyReadingEntity>, AppError> {
        let sql = "SELECT * FROM daily_readings WHERE id = $1";
        let row = sqlx::query_as::<_, DailyReadingEntity>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let sql = r#"
            DELETE FROM daily_readings
            WHERE id = $1
              AND EXISTS (
                  SELECT 1 FROM planting_sessions s
                  WHERE s.id = daily_readings.session_id AND s.is_locked = FALSE
              )
        "#;

        let result = sqlx::query(sql).bind(id).execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            // Check if reading exists but session is locked
            let exists: Option<(bool,)> = sqlx::query_as(
                r#"
                SELECT s.is_locked
                FROM daily_readings dr
                INNER JOIN planting_sessions s ON s.id = dr.session_id
                WHERE dr.id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some((is_locked,)) = exists {
                if is_locked {
                    return Err(AppError::BadRequest(
                        "Sesi tanam sudah selesai dan terkunci".to_string(),
                    ));
                }
            }

            return Err(AppError::NotFound(format!(
                "Catatan harian dengan ID '{id}' tidak ditemukan"
            )));
        }

        Ok(())
    }

    async fn find_unreported_active_sessions(
        &self,
        eval_date: NaiveDate,
        location_id: Option<&str>,
    ) -> Result<(i64, i64, i64, Vec<SessionRowWithRelations>), AppError> {
        // 1. Total active sessions
        let mut total_sql =
            "SELECT COUNT(*)::BIGINT FROM planting_sessions WHERE status = 'Active'".to_string();
        if location_id.is_some() {
            total_sql.push_str(" AND location_id = $1");
        }
        let mut total_q = sqlx::query_as::<_, (i64,)>(&total_sql);
        if let Some(loc) = location_id {
            total_q = total_q.bind(loc);
        }
        let (total_active_sessions,) = total_q.fetch_one(&self.pool).await?;

        // 2. Reported active sessions count
        let mut reported_sql = r#"
            SELECT COUNT(DISTINCT s.id)::BIGINT
            FROM planting_sessions s
            INNER JOIN daily_readings dr ON dr.session_id = s.id
            WHERE s.status = 'Active' AND dr.reading_date = $1
        "#
        .to_string();
        if location_id.is_some() {
            reported_sql.push_str(" AND s.location_id = $2");
        }
        let mut reported_q = sqlx::query_as::<_, (i64,)>(&reported_sql).bind(eval_date);
        if let Some(loc) = location_id {
            reported_q = reported_q.bind(loc);
        }
        let (reported_count,) = reported_q.fetch_one(&self.pool).await?;
        let unreported_count = (total_active_sessions - reported_count).max(0);

        // 3. Unreported active session list
        let mut list_sql = r#"
            SELECT
                s.id,
                s.session_name,
                s.plant_type_id,
                p.name AS plant_name,
                p.variety AS plant_variety,
                s.location_id,
                l.name AS location_name,
                s.installation_id,
                i.name AS installation_name,
                s.seeding_date,
                s.start_date,
                s.estimated_harvest_date,
                s.harvest_date,
                s.total_holes,
                s.status,
                s.is_locked,
                s.target_growth_est_days,
                s.target_veg_phase_days,
                s.target_yield_per_hole,
                s.target_min_ph,
                s.target_ideal_min_ph,
                s.target_ideal_max_ph,
                s.target_max_ph,
                s.target_ppm_veg_weeks,
                s.target_ppm_gen_weeks,
                s.target_ideal_min_ppm_veg,
                s.target_ideal_max_ppm_veg,
                s.target_tol_min_ppm_veg,
                s.target_tol_max_ppm_veg,
                s.target_ideal_min_ppm_gen,
                s.target_ideal_max_ppm_gen,
                s.target_tol_min_ppm_gen,
                s.target_tol_max_ppm_gen,
                s.target_ideal_min_temp_env,
                s.target_ideal_max_temp_env,
                s.target_ideal_min_temp_water,
                s.target_ideal_max_temp_water,
                s.target_ideal_min_hum,
                s.target_ideal_max_hum,
                s.actual_yield_kg,
                s.saleable_count,
                s.mortality_count,
                s.harvest_notes,
                s.notes,
                s.created_at,
                s.created_by,
                s.modify_at,
                s.modify_by,
                dm.total_daily_mortality,
                FALSE AS has_reported_today,
                lr.id AS latest_reading_id,
                lr.reading_date AS latest_reading_date,
                lr.reading_time AS latest_reading_time,
                lr.ppm AS latest_reading_ppm,
                lr.ph AS latest_reading_ph
            FROM planting_sessions s
            INNER JOIN plant_types p ON p.id = s.plant_type_id
            INNER JOIN locations l ON l.id = s.location_id
            LEFT JOIN installations i ON i.id = s.installation_id
            LEFT JOIN LATERAL (
                SELECT COALESCE(SUM(dr.mortality_count), 0)::BIGINT AS total_daily_mortality
                FROM daily_readings dr
                WHERE dr.session_id = s.id
            ) dm ON TRUE
            LEFT JOIN LATERAL (
                SELECT dr.id, dr.reading_date, dr.reading_time, dr.ppm, dr.ph
                FROM daily_readings dr
                WHERE dr.session_id = s.id
                ORDER BY dr.reading_date DESC, dr.reading_time DESC, dr.created_at DESC
                LIMIT 1
            ) lr ON TRUE
            WHERE s.status = 'Active'
              AND NOT EXISTS (
                  SELECT 1 FROM daily_readings dr
                  WHERE dr.session_id = s.id AND dr.reading_date = $1
              )
        "#
        .to_string();

        if location_id.is_some() {
            list_sql.push_str(" AND s.location_id = $2");
        }
        list_sql.push_str(" ORDER BY s.start_date DESC, s.created_at DESC");

        let mut list_q = sqlx::query_as::<_, SessionRowWithRelations>(&list_sql).bind(eval_date);
        if let Some(loc) = location_id {
            list_q = list_q.bind(loc);
        }
        let sessions = list_q.fetch_all(&self.pool).await?;

        Ok((
            total_active_sessions,
            reported_count,
            unreported_count,
            sessions,
        ))
    }
}
