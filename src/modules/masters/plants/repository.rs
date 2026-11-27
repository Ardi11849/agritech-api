use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::plants::dto::{PlantEntity, PlantListQuery};

#[async_trait]
pub trait PlantRepository: Send + Sync {
    async fn find_all(&self, query: &PlantListQuery) -> Result<Vec<PlantEntity>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<PlantEntity>, AppError>;
    async fn find_by_name_and_variety(
        &self,
        name: &str,
        variety: &str,
    ) -> Result<Option<PlantEntity>, AppError>;
    async fn create(&self, plant: &PlantEntity) -> Result<PlantEntity, AppError>;
    async fn update(&self, plant: &PlantEntity) -> Result<PlantEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgPlantRepository {
    pool: PgPool,
}

impl PgPlantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlantRepository for PgPlantRepository {
    async fn find_all(&self, query: &PlantListQuery) -> Result<Vec<PlantEntity>, AppError> {
        let mut sql = String::from(
            r#"
            SELECT
                id, name, variety, growth_est_days, veg_phase_days, target_yield_per_hole,
                min_ph, ideal_min_ph, ideal_max_ph, max_ph,
                ideal_ppm_veg_weeks, ideal_ppm_gen_weeks,
                ideal_min_ppm_veg, ideal_max_ppm_veg, tol_min_ppm_veg, tol_max_ppm_veg,
                ideal_min_ppm_gen, ideal_max_ppm_gen, tol_min_ppm_gen, tol_max_ppm_gen,
                ideal_min_temp_env, ideal_max_temp_env, ideal_min_temp_water, ideal_max_temp_water,
                ideal_min_hum, ideal_max_hum,
                is_active, created_at, created_by, modify_at, modify_by
            FROM plant_types
            WHERE 1=1
            "#,
        );

        if query.is_active.is_some() {
            sql.push_str(" AND is_active = $1");
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                if query.is_active.is_some() {
                    sql.push_str(" AND (name ILIKE $2 OR variety ILIKE $2)");
                } else {
                    sql.push_str(" AND (name ILIKE $1 OR variety ILIKE $1)");
                }
            }
        }

        sql.push_str(" ORDER BY name ASC, variety ASC");

        let mut q = sqlx::query_as::<_, PlantEntity>(&sql);

        if let Some(active) = query.is_active {
            q = q.bind(active);
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                q = q.bind(format!("%{}%", s.trim()));
            }
        }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<PlantEntity>, AppError> {
        let row = sqlx::query_as::<_, PlantEntity>(
            r#"
            SELECT
                id, name, variety, growth_est_days, veg_phase_days, target_yield_per_hole,
                min_ph, ideal_min_ph, ideal_max_ph, max_ph,
                ideal_ppm_veg_weeks, ideal_ppm_gen_weeks,
                ideal_min_ppm_veg, ideal_max_ppm_veg, tol_min_ppm_veg, tol_max_ppm_veg,
                ideal_min_ppm_gen, ideal_max_ppm_gen, tol_min_ppm_gen, tol_max_ppm_gen,
                ideal_min_temp_env, ideal_max_temp_env, ideal_min_temp_water, ideal_max_temp_water,
                ideal_min_hum, ideal_max_hum,
                is_active, created_at, created_by, modify_at, modify_by
            FROM plant_types
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_name_and_variety(
        &self,
        name: &str,
        variety: &str,
    ) -> Result<Option<PlantEntity>, AppError> {
        let row = sqlx::query_as::<_, PlantEntity>(
            r#"
            SELECT
                id, name, variety, growth_est_days, veg_phase_days, target_yield_per_hole,
                min_ph, ideal_min_ph, ideal_max_ph, max_ph,
                ideal_ppm_veg_weeks, ideal_ppm_gen_weeks,
                ideal_min_ppm_veg, ideal_max_ppm_veg, tol_min_ppm_veg, tol_max_ppm_veg,
                ideal_min_ppm_gen, ideal_max_ppm_gen, tol_min_ppm_gen, tol_max_ppm_gen,
                ideal_min_temp_env, ideal_max_temp_env, ideal_min_temp_water, ideal_max_temp_water,
                ideal_min_hum, ideal_max_hum,
                is_active, created_at, created_by, modify_at, modify_by
            FROM plant_types
            WHERE LOWER(name) = LOWER($1) AND LOWER(variety) = LOWER($2)
            "#,
        )
        .bind(name)
        .bind(variety)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn create(&self, plant: &PlantEntity) -> Result<PlantEntity, AppError> {
        sqlx::query(
            r#"
            INSERT INTO plant_types (
                id, name, variety, growth_est_days, veg_phase_days, target_yield_per_hole,
                min_ph, ideal_min_ph, ideal_max_ph, max_ph,
                ideal_ppm_veg_weeks, ideal_ppm_gen_weeks,
                ideal_min_ppm_veg, ideal_max_ppm_veg, tol_min_ppm_veg, tol_max_ppm_veg,
                ideal_min_ppm_gen, ideal_max_ppm_gen, tol_min_ppm_gen, tol_max_ppm_gen,
                ideal_min_temp_env, ideal_max_temp_env, ideal_min_temp_water, ideal_max_temp_water,
                ideal_min_hum, ideal_max_hum,
                is_active, created_at, created_by, modify_at, modify_by
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12,
                $13, $14, $15, $16,
                $17, $18, $19, $20,
                $21, $22, $23, $24,
                $25, $26,
                $27, $28, $29, $30, $31
            )
            "#,
        )
        .bind(&plant.id)
        .bind(&plant.name)
        .bind(&plant.variety)
        .bind(plant.growth_est_days)
        .bind(plant.veg_phase_days)
        .bind(plant.target_yield_per_hole)
        .bind(plant.min_ph)
        .bind(plant.ideal_min_ph)
        .bind(plant.ideal_max_ph)
        .bind(plant.max_ph)
        .bind(&plant.ideal_ppm_veg_weeks)
        .bind(&plant.ideal_ppm_gen_weeks)
        .bind(plant.ideal_min_ppm_veg)
        .bind(plant.ideal_max_ppm_veg)
        .bind(plant.tol_min_ppm_veg)
        .bind(plant.tol_max_ppm_veg)
        .bind(plant.ideal_min_ppm_gen)
        .bind(plant.ideal_max_ppm_gen)
        .bind(plant.tol_min_ppm_gen)
        .bind(plant.tol_max_ppm_gen)
        .bind(plant.ideal_min_temp_env)
        .bind(plant.ideal_max_temp_env)
        .bind(plant.ideal_min_temp_water)
        .bind(plant.ideal_max_temp_water)
        .bind(plant.ideal_min_hum)
        .bind(plant.ideal_max_hum)
        .bind(plant.is_active)
        .bind(plant.created_at)
        .bind(&plant.created_by)
        .bind(plant.modify_at)
        .bind(&plant.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(plant.clone())
    }

    async fn update(&self, plant: &PlantEntity) -> Result<PlantEntity, AppError> {
        sqlx::query(
            r#"
            UPDATE plant_types SET
                name = $2,
                variety = $3,
                growth_est_days = $4,
                veg_phase_days = $5,
                target_yield_per_hole = $6,
                min_ph = $7,
                ideal_min_ph = $8,
                ideal_max_ph = $9,
                max_ph = $10,
                ideal_ppm_veg_weeks = $11,
                ideal_ppm_gen_weeks = $12,
                ideal_min_ppm_veg = $13,
                ideal_max_ppm_veg = $14,
                tol_min_ppm_veg = $15,
                tol_max_ppm_veg = $16,
                ideal_min_ppm_gen = $17,
                ideal_max_ppm_gen = $18,
                tol_min_ppm_gen = $19,
                tol_max_ppm_gen = $20,
                ideal_min_temp_env = $21,
                ideal_max_temp_env = $22,
                ideal_min_temp_water = $23,
                ideal_max_temp_water = $24,
                ideal_min_hum = $25,
                ideal_max_hum = $26,
                is_active = $27,
                modify_at = $28,
                modify_by = $29
            WHERE id = $1
            "#,
        )
        .bind(&plant.id)
        .bind(&plant.name)
        .bind(&plant.variety)
        .bind(plant.growth_est_days)
        .bind(plant.veg_phase_days)
        .bind(plant.target_yield_per_hole)
        .bind(plant.min_ph)
        .bind(plant.ideal_min_ph)
        .bind(plant.ideal_max_ph)
        .bind(plant.max_ph)
        .bind(&plant.ideal_ppm_veg_weeks)
        .bind(&plant.ideal_ppm_gen_weeks)
        .bind(plant.ideal_min_ppm_veg)
        .bind(plant.ideal_max_ppm_veg)
        .bind(plant.tol_min_ppm_veg)
        .bind(plant.tol_max_ppm_veg)
        .bind(plant.ideal_min_ppm_gen)
        .bind(plant.ideal_max_ppm_gen)
        .bind(plant.tol_min_ppm_gen)
        .bind(plant.tol_max_ppm_gen)
        .bind(plant.ideal_min_temp_env)
        .bind(plant.ideal_max_temp_env)
        .bind(plant.ideal_min_temp_water)
        .bind(plant.ideal_max_temp_water)
        .bind(plant.ideal_min_hum)
        .bind(plant.ideal_max_hum)
        .bind(plant.is_active)
        .bind(plant.modify_at)
        .bind(&plant.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(plant.clone())
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM plant_types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
