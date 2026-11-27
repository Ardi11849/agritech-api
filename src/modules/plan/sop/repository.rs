use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use super::dto::*;
use crate::common::error::AppError;

#[async_trait]
pub trait SopRepository: Send + Sync {
    async fn find_all(&self, query: &SopQuery) -> Result<(SopKpi, Vec<SopResponse>), AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<SopResponse>, AppError>;
    async fn create(
        &self,
        sop: SopRow,
        steps: Vec<NormalizedSopStep>,
    ) -> Result<SopResponse, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn update(
        &self,
        id: &str,
        title: &str,
        category: &str,
        description: &str,
        status: &str,
        steps: Vec<NormalizedSopStep>,
        actor: &str,
    ) -> Result<SopResponse, AppError>;
    async fn delete(&self, id: &str) -> Result<bool, AppError>;
}

pub struct PgSopRepository {
    pool: PgPool,
}

impl PgSopRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SopRepository for PgSopRepository {
    async fn find_all(&self, query: &SopQuery) -> Result<(SopKpi, Vec<SopResponse>), AppError> {
        // 1. Ambil KPI dari seluruh data global SOP
        let kpi_row: (i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::BIGINT AS total_sops,
                COUNT(*) FILTER (WHERE status = 'Published')::BIGINT AS published_sops,
                COUNT(*) FILTER (WHERE status = 'Draft')::BIGINT AS draft_sops,
                COUNT(*) FILTER (WHERE status = 'Archived')::BIGINT AS archived_sops,
                (SELECT COUNT(*)::BIGINT FROM sop_steps) AS total_steps
            FROM sops
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let kpi = SopKpi {
            total_sops: kpi_row.0,
            published_sops: kpi_row.1,
            draft_sops: kpi_row.2,
            archived_sops: kpi_row.3,
            total_steps: kpi_row.4,
        };

        // 2. Filter data SOP
        let mut builder = sqlx::QueryBuilder::new(
            r#"
            SELECT id, title, category, description, version, status,
                   created_at, created_by, modify_at, modify_by
            FROM sops
            WHERE 1=1
            "#,
        );

        if let Some(cat) = &query.category {
            if !cat.trim().is_empty() && cat != "Semua" {
                builder.push(" AND category = ");
                builder.push_bind(cat.trim());
            }
        }

        if let Some(st) = &query.status {
            if !st.trim().is_empty() && st != "Semua" {
                builder.push(" AND status = ");
                builder.push_bind(st.trim());
            }
        }

        if let Some(search) = &query.search {
            let s = search.trim();
            if !s.is_empty() {
                let pattern = format!("%{s}%");
                builder.push(" AND (title ILIKE ");
                builder.push_bind(pattern.clone());
                builder.push(" OR description ILIKE ");
                builder.push_bind(pattern);
                builder.push(")");
            }
        }

        let sort_order = query.sort_order.as_deref().unwrap_or("desc");
        if sort_order.eq_ignore_ascii_case("asc") {
            builder.push(" ORDER BY modify_at ASC");
        } else {
            builder.push(" ORDER BY modify_at DESC");
        }

        let sop_rows: Vec<SopRow> = builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if sop_rows.is_empty() {
            return Ok((kpi, Vec::new()));
        }

        // 3. Batch fetch steps anti-N+1
        let sop_ids: Vec<String> = sop_rows.iter().map(|s| s.id.clone()).collect();
        let step_rows: Vec<SopStepRow> = sqlx::query_as(
            r#"
            SELECT id, sop_id, step_number, instruction, notes, created_at
            FROM sop_steps
            WHERE sop_id = ANY($1)
            ORDER BY sop_id, step_number ASC
            "#,
        )
        .bind(&sop_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut steps_by_sop: HashMap<String, Vec<SopStepResponse>> = HashMap::new();
        for step in step_rows {
            steps_by_sop
                .entry(step.sop_id.clone())
                .or_default()
                .push(SopStepResponse {
                    id: step.id,
                    sop_id: step.sop_id,
                    step_number: step.step_number,
                    instruction: step.instruction,
                    notes: step.notes,
                    created_at: step.created_at,
                });
        }

        let items = sop_rows
            .into_iter()
            .map(|s| {
                let steps = steps_by_sop.remove(&s.id).unwrap_or_default();
                let step_count = steps.len() as i32;
                SopResponse {
                    category: SopCategory::from_str(&s.category).unwrap_or(SopCategory::Umum),
                    status: SopStatus::from_str(&s.status).unwrap_or(SopStatus::Draft),
                    id: s.id,
                    title: s.title,
                    description: s.description,
                    version: s.version,
                    step_count,
                    steps,
                    created_at: s.created_at,
                    created_by: s.created_by,
                    modify_at: s.modify_at,
                    modify_by: s.modify_by,
                }
            })
            .collect();

        Ok((kpi, items))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<SopResponse>, AppError> {
        let sop_row: Option<SopRow> = sqlx::query_as(
            r#"
            SELECT id, title, category, description, version, status,
                   created_at, created_by, modify_at, modify_by
            FROM sops
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let s = match sop_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let step_rows: Vec<SopStepRow> = sqlx::query_as(
            r#"
            SELECT id, sop_id, step_number, instruction, notes, created_at
            FROM sop_steps
            WHERE sop_id = $1
            ORDER BY step_number ASC
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let steps: Vec<SopStepResponse> = step_rows
            .into_iter()
            .map(|step| SopStepResponse {
                id: step.id,
                sop_id: step.sop_id,
                step_number: step.step_number,
                instruction: step.instruction,
                notes: step.notes,
                created_at: step.created_at,
            })
            .collect();

        let step_count = steps.len() as i32;

        Ok(Some(SopResponse {
            category: SopCategory::from_str(&s.category).unwrap_or(SopCategory::Umum),
            status: SopStatus::from_str(&s.status).unwrap_or(SopStatus::Draft),
            id: s.id,
            title: s.title,
            description: s.description,
            version: s.version,
            step_count,
            steps,
            created_at: s.created_at,
            created_by: s.created_by,
            modify_at: s.modify_at,
            modify_by: s.modify_by,
        }))
    }

    async fn create(
        &self,
        sop: SopRow,
        steps: Vec<NormalizedSopStep>,
    ) -> Result<SopResponse, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO sops (id, title, category, description, version, status, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&sop.id)
        .bind(&sop.title)
        .bind(&sop.category)
        .bind(&sop.description)
        .bind(sop.version)
        .bind(&sop.status)
        .bind(sop.created_at)
        .bind(&sop.created_by)
        .bind(sop.modify_at)
        .bind(&sop.modify_by)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut step_responses = Vec::with_capacity(steps.len());

        for step in steps {
            let step_id = format!("stp_{}", Uuid::new_v4());
            let created_at = sop.created_at;

            sqlx::query(
                r#"
                INSERT INTO sop_steps (id, sop_id, step_number, instruction, notes, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(&step_id)
            .bind(&sop.id)
            .bind(step.step_number)
            .bind(&step.instruction)
            .bind(&step.notes)
            .bind(created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            step_responses.push(SopStepResponse {
                id: step_id,
                sop_id: sop.id.clone(),
                step_number: step.step_number,
                instruction: step.instruction,
                notes: step.notes,
                created_at,
            });
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let step_count = step_responses.len() as i32;

        Ok(SopResponse {
            category: SopCategory::from_str(&sop.category).unwrap_or(SopCategory::Umum),
            status: SopStatus::from_str(&sop.status).unwrap_or(SopStatus::Draft),
            id: sop.id,
            title: sop.title,
            description: sop.description,
            version: sop.version,
            step_count,
            steps: step_responses,
            created_at: sop.created_at,
            created_by: sop.created_by,
            modify_at: sop.modify_at,
            modify_by: sop.modify_by,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn update(
        &self,
        id: &str,
        title: &str,
        category: &str,
        description: &str,
        status: &str,
        steps: Vec<NormalizedSopStep>,
        actor: &str,
    ) -> Result<SopResponse, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 1. Lock baris sops dengan SELECT ... FOR UPDATE untuk mencegah race condition
        let current_version: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT version FROM sops WHERE id = $1 FOR UPDATE
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let old_ver = match current_version {
            Some(v) => v,
            None => return Err(AppError::NotFound(format!("SOP with ID '{id}' not found"))),
        };

        let new_ver = old_ver + 1;

        let sop_row: SopRow = sqlx::query_as(
            r#"
            UPDATE sops
            SET title = $2,
                category = $3,
                description = $4,
                version = $5,
                status = $6,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $7
            WHERE id = $1
            RETURNING id, title, category, description, version, status,
                      created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(category)
        .bind(description)
        .bind(new_ver)
        .bind(status)
        .bind(actor)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. Hapus langkah lama
        sqlx::query("DELETE FROM sop_steps WHERE sop_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 3. Masukkan langkah baru ternormalisasi
        let mut step_responses = Vec::with_capacity(steps.len());

        for step in steps {
            let step_id = format!("stp_{}", Uuid::new_v4());
            let created_at = sop_row.modify_at;

            sqlx::query(
                r#"
                INSERT INTO sop_steps (id, sop_id, step_number, instruction, notes, created_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(&step_id)
            .bind(id)
            .bind(step.step_number)
            .bind(&step.instruction)
            .bind(&step.notes)
            .bind(created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

            step_responses.push(SopStepResponse {
                id: step_id,
                sop_id: id.to_string(),
                step_number: step.step_number,
                instruction: step.instruction,
                notes: step.notes,
                created_at,
            });
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let step_count = step_responses.len() as i32;

        Ok(SopResponse {
            category: SopCategory::from_str(&sop_row.category).unwrap_or(SopCategory::Umum),
            status: SopStatus::from_str(&sop_row.status).unwrap_or(SopStatus::Draft),
            id: sop_row.id,
            title: sop_row.title,
            description: sop_row.description,
            version: sop_row.version,
            step_count,
            steps: step_responses,
            created_at: sop_row.created_at,
            created_by: sop_row.created_by,
            modify_at: sop_row.modify_at,
            modify_by: sop_row.modify_by,
        })
    }

    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let res = sqlx::query("DELETE FROM sops WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(res.rows_affected() > 0)
    }
}
