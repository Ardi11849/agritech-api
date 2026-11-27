use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::installations::dto::{
    InstallationEntity, InstallationListQuery, InstallationRow,
};

#[async_trait]
pub trait InstallationRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &InstallationListQuery,
    ) -> Result<Vec<InstallationRow>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<InstallationRow>, AppError>;
    async fn find_by_location_and_name(
        &self,
        location_id: &str,
        name: &str,
    ) -> Result<Option<InstallationEntity>, AppError>;
    async fn location_exists(&self, location_id: &str) -> Result<bool, AppError>;
    async fn get_location_name(&self, location_id: &str) -> Result<Option<String>, AppError>;
    async fn create(&self, ins: &InstallationEntity) -> Result<InstallationEntity, AppError>;
    async fn update(&self, ins: &InstallationEntity) -> Result<InstallationEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgInstallationRepository {
    pool: PgPool,
}

impl PgInstallationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InstallationRepository for PgInstallationRepository {
    async fn find_all(
        &self,
        query: &InstallationListQuery,
    ) -> Result<Vec<InstallationRow>, AppError> {
        let mut sql = String::from(
            r#"
            SELECT
                i.id, i.location_id, l.name AS location_name,
                i.name, i.capacity_holes, i.status, i.notes,
                i.created_at, i.created_by, i.modify_at, i.modify_by
            FROM installations i
            JOIN locations l ON l.id = i.location_id
            WHERE 1=1
            "#,
        );

        let mut param_idx = 1;
        let mut location_id_val = None;
        let mut status_val = None;
        let mut search_val = None;

        if let Some(ref lid) = query.location_id {
            if !lid.trim().is_empty() {
                sql.push_str(&format!(" AND i.location_id = ${param_idx}"));
                location_id_val = Some(lid.trim().to_string());
                param_idx += 1;
            }
        }

        if let Some(status) = query.status {
            sql.push_str(&format!(" AND i.status = ${param_idx}"));
            status_val = Some(status.to_string());
            param_idx += 1;
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                sql.push_str(&format!(
                    " AND (i.name ILIKE ${param_idx} OR l.name ILIKE ${param_idx})"
                ));
                search_val = Some(format!("%{}%", s.trim()));
            }
        }

        sql.push_str(" ORDER BY l.name ASC, i.name ASC");

        let mut q = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                i32,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
            ),
        >(&sql);

        if let Some(lid) = location_id_val {
            q = q.bind(lid);
        }

        if let Some(st) = status_val {
            q = q.bind(st);
        }

        if let Some(s) = search_val {
            q = q.bind(s);
        }

        let rows = q.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|r| InstallationRow {
                id: r.0,
                location_id: r.1,
                location_name: r.2,
                name: r.3,
                capacity_holes: r.4,
                status: r.5,
                notes: r.6,
                created_at: r.7,
                created_by: r.8,
                modify_at: r.9,
                modify_by: r.10,
            })
            .collect())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<InstallationRow>, AppError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                i32,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
            ),
        >(
            r#"
            SELECT
                i.id, i.location_id, l.name AS location_name,
                i.name, i.capacity_holes, i.status, i.notes,
                i.created_at, i.created_by, i.modify_at, i.modify_by
            FROM installations i
            JOIN locations l ON l.id = i.location_id
            WHERE i.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| InstallationRow {
            id: r.0,
            location_id: r.1,
            location_name: r.2,
            name: r.3,
            capacity_holes: r.4,
            status: r.5,
            notes: r.6,
            created_at: r.7,
            created_by: r.8,
            modify_at: r.9,
            modify_by: r.10,
        }))
    }

    async fn find_by_location_and_name(
        &self,
        location_id: &str,
        name: &str,
    ) -> Result<Option<InstallationEntity>, AppError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i32,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                Option<String>,
            ),
        >(
            r#"
            SELECT
                id, location_id, name, capacity_holes, status, notes,
                created_at, created_by, modify_at, modify_by
            FROM installations
            WHERE location_id = $1 AND LOWER(name) = LOWER($2)
            "#,
        )
        .bind(location_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| InstallationEntity {
            id: r.0,
            location_id: r.1,
            name: r.2,
            capacity_holes: r.3,
            status: r.4,
            notes: r.5,
            created_at: r.6,
            created_by: r.7,
            modify_at: r.8,
            modify_by: r.9,
        }))
    }

    async fn location_exists(&self, location_id: &str) -> Result<bool, AppError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM locations WHERE id = $1")
            .bind(location_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 > 0)
    }

    async fn get_location_name(&self, location_id: &str) -> Result<Option<String>, AppError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT name FROM locations WHERE id = $1")
            .bind(location_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.0))
    }

    async fn create(&self, ins: &InstallationEntity) -> Result<InstallationEntity, AppError> {
        sqlx::query(
            r#"
            INSERT INTO installations (
                id, location_id, name, capacity_holes, status, notes,
                created_at, created_by, modify_at, modify_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&ins.id)
        .bind(&ins.location_id)
        .bind(&ins.name)
        .bind(ins.capacity_holes)
        .bind(&ins.status)
        .bind(&ins.notes)
        .bind(ins.created_at)
        .bind(&ins.created_by)
        .bind(ins.modify_at)
        .bind(&ins.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(ins.clone())
    }

    async fn update(&self, ins: &InstallationEntity) -> Result<InstallationEntity, AppError> {
        sqlx::query(
            r#"
            UPDATE installations SET
                location_id = $2,
                name = $3,
                capacity_holes = $4,
                status = $5,
                notes = $6,
                modify_at = $7,
                modify_by = $8
            WHERE id = $1
            "#,
        )
        .bind(&ins.id)
        .bind(&ins.location_id)
        .bind(&ins.name)
        .bind(ins.capacity_holes)
        .bind(&ins.status)
        .bind(&ins.notes)
        .bind(ins.modify_at)
        .bind(&ins.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(ins.clone())
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM installations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
