use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

use super::dto::*;
use crate::common::error::AppError;

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_all(&self, query: &TaskQuery) -> Result<Vec<TaskJoinedRow>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<TaskJoinedRow>, AppError>;
    async fn create(&self, row: TaskJoinedRow) -> Result<TaskJoinedRow, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn update(
        &self,
        id: &str,
        title: &str,
        description: &str,
        status: &str,
        priority: &str,
        story_points: i32,
        due_date: Option<NaiveDate>,
        assigned_to: Option<&str>,
        session_id: Option<&str>,
        sop_id: Option<&str>,
        completed_at: Option<DateTime<Utc>>,
        actor: &str,
    ) -> Result<TaskJoinedRow, AppError>;
    async fn update_status(
        &self,
        id: &str,
        status: &str,
        story_points: Option<i32>,
        completed_at: Option<DateTime<Utc>>,
        actor: &str,
    ) -> Result<TaskJoinedRow, AppError>;
    async fn delete(&self, id: &str) -> Result<bool, AppError>;

    async fn check_user_exists(&self, id: &str) -> Result<bool, AppError>;
    async fn check_session_exists(&self, id: &str) -> Result<bool, AppError>;
    async fn check_sop_exists(&self, id: &str) -> Result<bool, AppError>;
}

pub struct PgTaskRepository {
    pool: PgPool,
}

impl PgTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for PgTaskRepository {
    async fn find_all(&self, query: &TaskQuery) -> Result<Vec<TaskJoinedRow>, AppError> {
        let mut builder = sqlx::QueryBuilder::new(
            r#"
            SELECT
                t.id, t.title, t.description, t.status, t.priority, t.story_points, t.due_date,
                t.assigned_to, u.username AS assigned_username, u.name AS assigned_name,
                t.session_id, s.session_name AS session_name,
                t.sop_id, sp.title AS sop_title,
                t.completed_at, t.created_at, t.created_by, t.modify_at, t.modify_by
            FROM farm_tasks t
            LEFT JOIN users u ON u.id = t.assigned_to
            LEFT JOIN planting_sessions s ON s.id = t.session_id
            LEFT JOIN sops sp ON sp.id = t.sop_id
            WHERE 1=1
            "#,
        );

        if let Some(status) = &query.status {
            if !status.trim().is_empty() && status != "Semua" {
                builder.push(" AND t.status = ");
                builder.push_bind(status.trim());
            }
        }

        if let Some(priority) = &query.priority {
            if !priority.trim().is_empty() && priority != "Semua" {
                builder.push(" AND t.priority = ");
                builder.push_bind(priority.trim());
            }
        }

        if let Some(assigned) = &query.assigned_to {
            if !assigned.trim().is_empty() && assigned != "Semua" {
                builder.push(" AND t.assigned_to = ");
                builder.push_bind(assigned.trim());
            }
        }

        if let Some(sess) = &query.session_id {
            if !sess.trim().is_empty() && sess != "Semua" {
                builder.push(" AND t.session_id = ");
                builder.push_bind(sess.trim());
            }
        }

        if let Some(sop) = &query.sop_id {
            if !sop.trim().is_empty() && sop != "Semua" {
                builder.push(" AND t.sop_id = ");
                builder.push_bind(sop.trim());
            }
        }

        if let Some(search) = &query.search {
            let s = search.trim();
            if !s.is_empty() {
                let pattern = format!("%{s}%");
                builder.push(" AND (t.title ILIKE ");
                builder.push_bind(pattern.clone());
                builder.push(" OR t.description ILIKE ");
                builder.push_bind(pattern.clone());
                builder.push(" OR COALESCE(u.name, '') ILIKE ");
                builder.push_bind(pattern.clone());
                builder.push(" OR COALESCE(u.username, '') ILIKE ");
                builder.push_bind(pattern.clone());
                builder.push(" OR COALESCE(s.session_name, '') ILIKE ");
                builder.push_bind(pattern.clone());
                builder.push(" OR COALESCE(sp.title, '') ILIKE ");
                builder.push_bind(pattern);
                builder.push(")");
            }
        }

        let sort_order = query.sort_order.as_deref().unwrap_or("asc");
        if sort_order.eq_ignore_ascii_case("desc") {
            builder.push(" ORDER BY t.due_date DESC NULLS LAST, t.created_at DESC");
        } else {
            builder.push(" ORDER BY t.due_date ASC NULLS LAST, t.created_at DESC");
        }

        let rows: Vec<TaskJoinedRow> = builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<TaskJoinedRow>, AppError> {
        let sql = r#"
            SELECT
                t.id, t.title, t.description, t.status, t.priority, t.story_points, t.due_date,
                t.assigned_to, u.username AS assigned_username, u.name AS assigned_name,
                t.session_id, s.session_name AS session_name,
                t.sop_id, sp.title AS sop_title,
                t.completed_at, t.created_at, t.created_by, t.modify_at, t.modify_by
            FROM farm_tasks t
            LEFT JOIN users u ON u.id = t.assigned_to
            LEFT JOIN planting_sessions s ON s.id = t.session_id
            LEFT JOIN sops sp ON sp.id = t.sop_id
            WHERE t.id = $1
        "#;

        let row: Option<TaskJoinedRow> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row)
    }

    async fn create(&self, row: TaskJoinedRow) -> Result<TaskJoinedRow, AppError> {
        sqlx::query(
            r#"
            INSERT INTO farm_tasks (
                id, title, description, status, priority, story_points, due_date,
                assigned_to, session_id, sop_id, completed_at, created_at, created_by, modify_at, modify_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&row.id)
        .bind(&row.title)
        .bind(&row.description)
        .bind(&row.status)
        .bind(&row.priority)
        .bind(row.story_points)
        .bind(row.due_date)
        .bind(&row.assigned_to)
        .bind(&row.session_id)
        .bind(&row.sop_id)
        .bind(row.completed_at)
        .bind(row.created_at)
        .bind(&row.created_by)
        .bind(row.modify_at)
        .bind(&row.modify_by)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        self.find_by_id(&row.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to load newly created task".to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn update(
        &self,
        id: &str,
        title: &str,
        description: &str,
        status: &str,
        priority: &str,
        story_points: i32,
        due_date: Option<NaiveDate>,
        assigned_to: Option<&str>,
        session_id: Option<&str>,
        sop_id: Option<&str>,
        completed_at: Option<DateTime<Utc>>,
        actor: &str,
    ) -> Result<TaskJoinedRow, AppError> {
        let res = sqlx::query(
            r#"
            UPDATE farm_tasks
            SET title = $2,
                description = $3,
                status = $4,
                priority = $5,
                story_points = $6,
                due_date = $7,
                assigned_to = $8,
                session_id = $9,
                sop_id = $10,
                completed_at = $11,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $12
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(description)
        .bind(status)
        .bind(priority)
        .bind(story_points)
        .bind(due_date)
        .bind(assigned_to)
        .bind(session_id)
        .bind(sop_id)
        .bind(completed_at)
        .bind(actor)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Task with ID '{id}' not found")));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to load updated task".to_string()))
    }

    async fn update_status(
        &self,
        id: &str,
        status: &str,
        story_points: Option<i32>,
        completed_at: Option<DateTime<Utc>>,
        actor: &str,
    ) -> Result<TaskJoinedRow, AppError> {
        let res = match story_points {
            Some(sp) => {
                sqlx::query(
                    r#"
                    UPDATE farm_tasks
                    SET status = $2,
                        story_points = $3,
                        completed_at = $4,
                        modify_at = CURRENT_TIMESTAMP,
                        modify_by = $5
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(status)
                .bind(sp)
                .bind(completed_at)
                .bind(actor)
                .execute(&self.pool)
                .await
            }
            None => {
                sqlx::query(
                    r#"
                    UPDATE farm_tasks
                    SET status = $2,
                        completed_at = $3,
                        modify_at = CURRENT_TIMESTAMP,
                        modify_by = $4
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(status)
                .bind(completed_at)
                .bind(actor)
                .execute(&self.pool)
                .await
            }
        }
        .map_err(|e| AppError::Database(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Task with ID '{id}' not found")));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to load updated task".to_string()))
    }

    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let res = sqlx::query("DELETE FROM farm_tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(res.rows_affected() > 0)
    }

    async fn check_user_exists(&self, id: &str) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*)::BIGINT FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count.0 > 0)
    }

    async fn check_session_exists(&self, id: &str) -> Result<bool, AppError> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*)::BIGINT FROM planting_sessions WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count.0 > 0)
    }

    async fn check_sop_exists(&self, id: &str) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*)::BIGINT FROM sops WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count.0 > 0)
    }
}
