use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::sessions::dto::{
    SessionEntity, SessionKpi, SessionListQuery, SessionRowWithRelations,
};

pub struct CompleteHarvestParams<'a> {
    pub harvest_date: NaiveDate,
    pub actual_yield_kg: Decimal,
    pub saleable_count: i32,
    pub mortality_count: i32,
    pub harvest_notes: &'a str,
    pub modify_by: &'a str,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &SessionListQuery,
        eval_date: NaiveDate,
    ) -> Result<(SessionKpi, Vec<SessionRowWithRelations>), AppError>;

    async fn find_by_id(
        &self,
        id: &str,
        eval_date: NaiveDate,
    ) -> Result<Option<SessionRowWithRelations>, AppError>;

    async fn find_raw_by_id(&self, id: &str) -> Result<Option<SessionEntity>, AppError>;

    async fn create(&self, entity: &SessionEntity) -> Result<(), AppError>;

    async fn update(&self, entity: &SessionEntity) -> Result<(), AppError>;

    async fn complete_harvest(
        &self,
        id: &str,
        params: &CompleteHarvestParams<'_>,
    ) -> Result<(), AppError>;

    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionKpiRow {
    active_sessions: i64,
    active_total_holes: i64,
    active_estimated_yield_kg: Decimal,
    completed_sessions: i64,
    unreported_today_count: i64,
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn find_all(
        &self,
        query: &SessionListQuery,
        eval_date: NaiveDate,
    ) -> Result<(SessionKpi, Vec<SessionRowWithRelations>), AppError> {
        // 1. Fetch KPI
        let kpi_row: SessionKpiRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'Active')::BIGINT AS active_sessions,
                COALESCE(SUM(total_holes) FILTER (WHERE status = 'Active'), 0)::BIGINT AS active_total_holes,
                COALESCE(SUM((total_holes::NUMERIC * target_yield_per_hole) / 1000.0) FILTER (WHERE status = 'Active'), 0.00)::NUMERIC AS active_estimated_yield_kg,
                COUNT(*) FILTER (WHERE status = 'Completed')::BIGINT AS completed_sessions,
                COUNT(*) FILTER (WHERE status = 'Active' AND NOT EXISTS (
                    SELECT 1 FROM daily_readings dr WHERE dr.session_id = planting_sessions.id AND dr.reading_date = $1
                ))::BIGINT AS unreported_today_count
            FROM planting_sessions;
            "#,
        )
        .bind(eval_date)
        .fetch_one(&self.pool)
        .await?;

        let kpi = SessionKpi {
            active_sessions: kpi_row.active_sessions,
            active_total_holes: kpi_row.active_total_holes,
            active_estimated_yield_kg: kpi_row.active_estimated_yield_kg.round_dp(2),
            completed_sessions: kpi_row.completed_sessions,
            unreported_today_count: kpi_row.unreported_today_count,
        };

        // 2. Build filtered sessions query
        let sort_order_clause = match query.sort_order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC",
        };

        let mut sql = r#"
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
                rt.has_reported_today,
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
                SELECT EXISTS (
                    SELECT 1 FROM daily_readings dr
                    WHERE dr.session_id = s.id AND dr.reading_date = $1
                ) AS has_reported_today
            ) rt ON TRUE
            LEFT JOIN LATERAL (
                SELECT dr.id, dr.reading_date, dr.reading_time, dr.ppm, dr.ph
                FROM daily_readings dr
                WHERE dr.session_id = s.id
                ORDER BY dr.reading_date DESC, dr.reading_time DESC, dr.created_at DESC
                LIMIT 1
            ) lr ON TRUE
            WHERE 1=1
            "#
        .to_string();

        let mut param_idx = 2;
        if query.location_id.is_some() {
            sql.push_str(&format!(" AND s.location_id = ${param_idx}"));
            param_idx += 1;
        }
        if query.installation_id.is_some() {
            sql.push_str(&format!(" AND s.installation_id = ${param_idx}"));
            param_idx += 1;
        }
        if query.plant_type_id.is_some() {
            sql.push_str(&format!(" AND s.plant_type_id = ${param_idx}"));
            param_idx += 1;
        }
        if query.status.is_some() {
            sql.push_str(&format!(" AND s.status = ${param_idx}"));
            param_idx += 1;
        }
        if query.search.is_some() {
            sql.push_str(&format!(
                " AND (s.session_name ILIKE ${idx} OR p.name ILIKE ${idx} OR l.name ILIKE ${idx} OR i.name ILIKE ${idx})",
                idx = param_idx
            ));
        }

        sql.push_str(&format!(
            " ORDER BY s.start_date {sort_order_clause}, s.created_at DESC"
        ));

        let mut q = sqlx::query_as::<_, SessionRowWithRelations>(&sql).bind(eval_date);

        if let Some(loc_id) = &query.location_id {
            q = q.bind(loc_id);
        }
        if let Some(ins_id) = &query.installation_id {
            q = q.bind(ins_id);
        }
        if let Some(plt_id) = &query.plant_type_id {
            q = q.bind(plt_id);
        }
        if let Some(st) = &query.status {
            q = q.bind(st.to_string());
        }
        if let Some(search) = &query.search {
            let pattern = format!("%{}%", search.trim());
            q = q.bind(pattern);
        }

        let items = q.fetch_all(&self.pool).await?;
        Ok((kpi, items))
    }

    async fn find_by_id(
        &self,
        id: &str,
        eval_date: NaiveDate,
    ) -> Result<Option<SessionRowWithRelations>, AppError> {
        let sql = r#"
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
                rt.has_reported_today,
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
                SELECT EXISTS (
                    SELECT 1 FROM daily_readings dr
                    WHERE dr.session_id = s.id AND dr.reading_date = $2
                ) AS has_reported_today
            ) rt ON TRUE
            LEFT JOIN LATERAL (
                SELECT dr.id, dr.reading_date, dr.reading_time, dr.ppm, dr.ph
                FROM daily_readings dr
                WHERE dr.session_id = s.id
                ORDER BY dr.reading_date DESC, dr.reading_time DESC, dr.created_at DESC
                LIMIT 1
            ) lr ON TRUE
            WHERE s.id = $1
            "#;

        let row = sqlx::query_as::<_, SessionRowWithRelations>(sql)
            .bind(id)
            .bind(eval_date)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    async fn find_raw_by_id(&self, id: &str) -> Result<Option<SessionEntity>, AppError> {
        let sql = "SELECT * FROM planting_sessions WHERE id = $1";
        let row = sqlx::query_as::<_, SessionEntity>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    async fn create(&self, entity: &SessionEntity) -> Result<(), AppError> {
        let sql = r#"
            INSERT INTO planting_sessions (
                id, session_name, plant_type_id, location_id, installation_id,
                seeding_date, start_date, estimated_harvest_date, harvest_date,
                total_holes, status, is_locked,
                target_growth_est_days, target_veg_phase_days, target_yield_per_hole,
                target_min_ph, target_ideal_min_ph, target_ideal_max_ph, target_max_ph,
                target_ppm_veg_weeks, target_ppm_gen_weeks,
                target_ideal_min_ppm_veg, target_ideal_max_ppm_veg,
                target_tol_min_ppm_veg, target_tol_max_ppm_veg,
                target_ideal_min_ppm_gen, target_ideal_max_ppm_gen,
                target_tol_min_ppm_gen, target_tol_max_ppm_gen,
                target_ideal_min_temp_env, target_ideal_max_temp_env,
                target_ideal_min_temp_water, target_ideal_max_temp_water,
                target_ideal_min_hum, target_ideal_max_hum,
                actual_yield_kg, saleable_count, mortality_count, harvest_notes,
                notes, created_at, created_by, modify_at, modify_by
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15,
                $16, $17, $18, $19,
                $20, $21,
                $22, $23,
                $24, $25,
                $26, $27,
                $28, $29,
                $30, $31,
                $32, $33,
                $34, $35,
                $36, $37, $38, $39,
                $40, $41, $42, $43, $44
            )
        "#;

        sqlx::query(sql)
            .bind(&entity.id)
            .bind(&entity.session_name)
            .bind(&entity.plant_type_id)
            .bind(&entity.location_id)
            .bind(&entity.installation_id)
            .bind(entity.seeding_date)
            .bind(entity.start_date)
            .bind(entity.estimated_harvest_date)
            .bind(entity.harvest_date)
            .bind(entity.total_holes)
            .bind(&entity.status)
            .bind(entity.is_locked)
            .bind(entity.target_growth_est_days)
            .bind(entity.target_veg_phase_days)
            .bind(entity.target_yield_per_hole)
            .bind(entity.target_min_ph)
            .bind(entity.target_ideal_min_ph)
            .bind(entity.target_ideal_max_ph)
            .bind(entity.target_max_ph)
            .bind(&entity.target_ppm_veg_weeks)
            .bind(&entity.target_ppm_gen_weeks)
            .bind(entity.target_ideal_min_ppm_veg)
            .bind(entity.target_ideal_max_ppm_veg)
            .bind(entity.target_tol_min_ppm_veg)
            .bind(entity.target_tol_max_ppm_veg)
            .bind(entity.target_ideal_min_ppm_gen)
            .bind(entity.target_ideal_max_ppm_gen)
            .bind(entity.target_tol_min_ppm_gen)
            .bind(entity.target_tol_max_ppm_gen)
            .bind(entity.target_ideal_min_temp_env)
            .bind(entity.target_ideal_max_temp_env)
            .bind(entity.target_ideal_min_temp_water)
            .bind(entity.target_ideal_max_temp_water)
            .bind(entity.target_ideal_min_hum)
            .bind(entity.target_ideal_max_hum)
            .bind(entity.actual_yield_kg)
            .bind(entity.saleable_count)
            .bind(entity.mortality_count)
            .bind(&entity.harvest_notes)
            .bind(&entity.notes)
            .bind(entity.created_at)
            .bind(&entity.created_by)
            .bind(entity.modify_at)
            .bind(&entity.modify_by)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update(&self, entity: &SessionEntity) -> Result<(), AppError> {
        let sql = r#"
            UPDATE planting_sessions SET
                session_name = $1,
                location_id = $2,
                installation_id = $3,
                seeding_date = $4,
                start_date = $5,
                estimated_harvest_date = $6,
                total_holes = $7,
                status = $8,
                notes = $9,
                modify_at = $10,
                modify_by = $11
            WHERE id = $12 AND is_locked = FALSE
        "#;

        let result = sqlx::query(sql)
            .bind(&entity.session_name)
            .bind(&entity.location_id)
            .bind(&entity.installation_id)
            .bind(entity.seeding_date)
            .bind(entity.start_date)
            .bind(entity.estimated_harvest_date)
            .bind(entity.total_holes)
            .bind(&entity.status)
            .bind(&entity.notes)
            .bind(entity.modify_at)
            .bind(&entity.modify_by)
            .bind(&entity.id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest(
                "Sesi tanam sudah selesai dan terkunci atau tidak ditemukan".to_string(),
            ));
        }

        Ok(())
    }

    async fn complete_harvest(
        &self,
        id: &str,
        params: &CompleteHarvestParams<'_>,
    ) -> Result<(), AppError> {
        let sql = r#"
            UPDATE planting_sessions SET
                harvest_date = $1,
                actual_yield_kg = $2,
                saleable_count = $3,
                mortality_count = $4,
                harvest_notes = $5,
                status = 'Completed',
                is_locked = TRUE,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $6
            WHERE id = $7 AND is_locked = FALSE
        "#;

        let result = sqlx::query(sql)
            .bind(params.harvest_date)
            .bind(params.actual_yield_kg)
            .bind(params.saleable_count)
            .bind(params.mortality_count)
            .bind(params.harvest_notes)
            .bind(params.modify_by)
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest(
                "Sesi tanam sudah selesai dan terkunci atau tidak ditemukan".to_string(),
            ));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let sql = "DELETE FROM planting_sessions WHERE id = $1 AND is_locked = FALSE";
        let result = sqlx::query(sql).bind(id).execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest(
                "Sesi tanam yang sudah selesai dan terkunci tidak dapat dihapus".to_string(),
            ));
        }

        Ok(())
    }
}
