use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::locations::dto::{
    LocationEntity, LocationListQuery, LocationSummaryRow,
};

#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &LocationListQuery,
    ) -> Result<Vec<LocationSummaryRow>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<LocationSummaryRow>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<LocationEntity>, AppError>;
    async fn count_installations_by_location(&self, location_id: &str) -> Result<i64, AppError>;
    async fn create(&self, loc: &LocationEntity) -> Result<LocationEntity, AppError>;
    async fn update(&self, loc: &LocationEntity) -> Result<LocationEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgLocationRepository {
    pool: PgPool,
}

impl PgLocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocationRepository for PgLocationRepository {
    async fn find_all(
        &self,
        query: &LocationListQuery,
    ) -> Result<Vec<LocationSummaryRow>, AppError> {
        let mut sql = String::from(
            r#"
            SELECT
                l.id, l.name, l.manager_name, l.address, l.is_active,
                l.created_at, l.created_by, l.modify_at, l.modify_by,
                COUNT(i.id)::BIGINT AS installation_count,
                COUNT(i.id) FILTER (WHERE i.status = 'Active')::BIGINT AS active_installation_count,
                COALESCE(SUM(i.capacity_holes), 0)::BIGINT AS total_capacity_holes,
                COALESCE(SUM(i.capacity_holes) FILTER (WHERE i.status = 'Active'), 0)::BIGINT AS active_capacity_holes
            FROM locations l
            LEFT JOIN installations i ON i.location_id = l.id
            WHERE 1=1
            "#,
        );

        if query.is_active.is_some() {
            sql.push_str(" AND l.is_active = $1");
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                if query.is_active.is_some() {
                    sql.push_str(
                        " AND (l.name ILIKE $2 OR l.manager_name ILIKE $2 OR l.address ILIKE $2)",
                    );
                } else {
                    sql.push_str(
                        " AND (l.name ILIKE $1 OR l.manager_name ILIKE $1 OR l.address ILIKE $1)",
                    );
                }
            }
        }

        sql.push_str(" GROUP BY l.id ORDER BY l.name ASC");

        let mut q = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                bool,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                i64,
                i64,
                i64,
                i64,
            ),
        >(&sql);

        if let Some(active) = query.is_active {
            q = q.bind(active);
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                q = q.bind(format!("%{}%", s.trim()));
            }
        }

        let rows = q.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|r| LocationSummaryRow {
                id: r.0,
                name: r.1,
                manager_name: r.2,
                address: r.3,
                is_active: r.4,
                created_at: r.5,
                created_by: r.6,
                modify_at: r.7,
                modify_by: r.8,
                installation_count: r.9,
                active_installation_count: r.10,
                total_capacity_holes: r.11,
                active_capacity_holes: r.12,
            })
            .collect())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<LocationSummaryRow>, AppError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                bool,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                i64,
                i64,
                i64,
                i64,
            ),
        >(
            r#"
            SELECT
                l.id, l.name, l.manager_name, l.address, l.is_active,
                l.created_at, l.created_by, l.modify_at, l.modify_by,
                COUNT(i.id)::BIGINT AS installation_count,
                COUNT(i.id) FILTER (WHERE i.status = 'Active')::BIGINT AS active_installation_count,
                COALESCE(SUM(i.capacity_holes), 0)::BIGINT AS total_capacity_holes,
                COALESCE(SUM(i.capacity_holes) FILTER (WHERE i.status = 'Active'), 0)::BIGINT AS active_capacity_holes
            FROM locations l
            LEFT JOIN installations i ON i.location_id = l.id
            WHERE l.id = $1
            GROUP BY l.id
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| LocationSummaryRow {
            id: r.0,
            name: r.1,
            manager_name: r.2,
            address: r.3,
            is_active: r.4,
            created_at: r.5,
            created_by: r.6,
            modify_at: r.7,
            modify_by: r.8,
            installation_count: r.9,
            active_installation_count: r.10,
            total_capacity_holes: r.11,
            active_capacity_holes: r.12,
        }))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<LocationEntity>, AppError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                bool,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
            ),
        >(
            r#"
            SELECT
                id, name, manager_name, address, is_active,
                created_at, created_by, modify_at, modify_by
            FROM locations
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| LocationEntity {
            id: r.0,
            name: r.1,
            manager_name: r.2,
            address: r.3,
            is_active: r.4,
            created_at: r.5,
            created_by: r.6,
            modify_at: r.7,
            modify_by: r.8,
        }))
    }

    async fn count_installations_by_location(&self, location_id: &str) -> Result<i64, AppError> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM installations WHERE location_id = $1")
                .bind(location_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    async fn create(&self, loc: &LocationEntity) -> Result<LocationEntity, AppError> {
        sqlx::query(
            r#"
            INSERT INTO locations (
                id, name, manager_name, address, is_active,
                created_at, created_by, modify_at, modify_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&loc.id)
        .bind(&loc.name)
        .bind(&loc.manager_name)
        .bind(&loc.address)
        .bind(loc.is_active)
        .bind(loc.created_at)
        .bind(&loc.created_by)
        .bind(loc.modify_at)
        .bind(&loc.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(loc.clone())
    }

    async fn update(&self, loc: &LocationEntity) -> Result<LocationEntity, AppError> {
        sqlx::query(
            r#"
            UPDATE locations SET
                name = $2,
                manager_name = $3,
                address = $4,
                is_active = $5,
                modify_at = $6,
                modify_by = $7
            WHERE id = $1
            "#,
        )
        .bind(&loc.id)
        .bind(&loc.name)
        .bind(&loc.manager_name)
        .bind(&loc.address)
        .bind(loc.is_active)
        .bind(loc.modify_at)
        .bind(&loc.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(loc.clone())
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM locations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
