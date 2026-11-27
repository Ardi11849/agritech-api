use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, QueryBuilder};

use crate::common::error::AppError;
use crate::modules::reports::audit::dto::NewAuditEventRow;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: String,
    pub user_id: Option<String>,
    pub username: String,
    pub action: String,
    pub module: String,
    pub target_id: String,
    pub description: String,
    pub old_data: Option<Value>,
    pub new_data: Option<Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NormalizedAuditFilter {
    pub modules: Vec<String>,
    pub action: Option<String>,
    pub username: Option<String>,
    pub search: Option<String>,
    pub start_ts: Option<DateTime<Utc>>,
    pub end_ts_exclusive: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn find_logs(
        &self,
        filter: &NormalizedAuditFilter,
    ) -> Result<(i64, Vec<AuditLogRow>), AppError>;

    async fn find_distinct_modules(&self) -> Result<Vec<String>, AppError>;

    async fn insert_event(&self, input: &NewAuditEventRow) -> Result<(), AppError>;
}

pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn find_logs(
        &self,
        filter: &NormalizedAuditFilter,
    ) -> Result<(i64, Vec<AuditLogRow>), AppError> {
        // 1. Count Query
        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM audit_logs WHERE 1=1");
        apply_filter_predicates(&mut count_builder, filter);

        let total_count: (i64,) = count_builder.build_query_as().fetch_one(&self.pool).await?;

        // 2. Data Query
        let mut data_builder = QueryBuilder::new(
            r#"
            SELECT
                id, user_id, username, action, module, target_id,
                description, old_data, new_data, ip_address, created_at
            FROM audit_logs
            WHERE 1=1
            "#,
        );
        apply_filter_predicates(&mut data_builder, filter);

        data_builder.push(" ORDER BY created_at DESC, id DESC LIMIT ");
        data_builder.push_bind(filter.limit);
        data_builder.push(" OFFSET ");
        data_builder.push_bind(filter.offset);

        let items: Vec<AuditLogRow> = data_builder.build_query_as().fetch_all(&self.pool).await?;

        Ok((total_count.0, items))
    }

    async fn find_distinct_modules(&self) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT module FROM audit_logs WHERE TRIM(module) <> '' ORDER BY module ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn insert_event(&self, input: &NewAuditEventRow) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, user_id, username, action, module, target_id,
                description, old_data, new_data, ip_address, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, CURRENT_TIMESTAMP
            )
            "#,
        )
        .bind(&input.id)
        .bind(&input.user_id)
        .bind(&input.username)
        .bind(&input.action)
        .bind(&input.module)
        .bind(&input.target_id)
        .bind(&input.description)
        .bind(&input.old_data)
        .bind(&input.new_data)
        .bind(&input.ip_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn apply_filter_predicates<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    filter: &'a NormalizedAuditFilter,
) {
    if !filter.modules.is_empty() {
        builder.push(" AND UPPER(module) = ANY(");
        builder.push_bind(&filter.modules);
        builder.push(")");
    }

    if let Some(act) = &filter.action {
        builder.push(" AND action = ");
        builder.push_bind(act);
    }

    if let Some(user) = &filter.username {
        builder.push(" AND username ILIKE ");
        let pattern = format!("%{user}%");
        builder.push_bind(pattern);
    }

    if let Some(search) = &filter.search {
        builder.push(" AND (description ILIKE ");
        let pattern = format!("%{search}%");
        builder.push_bind(pattern.clone());
        builder.push(" OR target_id ILIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR username ILIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR module ILIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }

    if let Some(start) = filter.start_ts {
        builder.push(" AND created_at >= ");
        builder.push_bind(start);
    }

    if let Some(end) = filter.end_ts_exclusive {
        builder.push(" AND created_at < ");
        builder.push_bind(end);
    }
}
