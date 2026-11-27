use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use super::dto::*;
use super::repository::TaskRepository;
use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;

pub fn is_task_overdue(status: TaskStatus, due_date: Option<NaiveDate>, today: NaiveDate) -> bool {
    status != TaskStatus::Complete && due_date.is_some_and(|d| d < today)
}

pub fn calculate_task_kpi(
    tasks: &[(TaskStatus, Option<NaiveDate>, i32)],
    today: NaiveDate,
) -> TaskKpi {
    let total_tasks = tasks.len() as i64;
    let mut completed_tasks = 0i64;
    let mut overdue_tasks = 0i64;
    let mut total_story_points = 0i32;
    let mut completed_story_points = 0i32;

    for (status, due_date, sp) in tasks {
        total_story_points += *sp;
        if *status == TaskStatus::Complete {
            completed_tasks += 1;
            completed_story_points += *sp;
        } else if is_task_overdue(*status, *due_date, today) {
            overdue_tasks += 1;
        }
    }

    let completion_rate_percent = if total_tasks == 0 {
        Decimal::ZERO
    } else {
        round_to_two_decimals(
            (Decimal::from(completed_tasks) * Decimal::from(100)) / Decimal::from(total_tasks),
        )
    };

    let story_point_velocity_percent = if total_story_points == 0 {
        Decimal::ZERO
    } else {
        round_to_two_decimals(
            (Decimal::from(completed_story_points) * Decimal::from(100))
                / Decimal::from(total_story_points),
        )
    };

    TaskKpi {
        total_tasks,
        completed_tasks,
        overdue_tasks,
        completion_rate_percent,
        total_story_points,
        completed_story_points,
        story_point_velocity_percent,
    }
}

#[async_trait]
pub trait TaskService: Send + Sync {
    async fn get_tasks(&self, query: &TaskQuery) -> Result<TaskListResponse, AppError>;
    async fn get_task_by_id(
        &self,
        id: &str,
        ref_date: Option<NaiveDate>,
    ) -> Result<TaskResponse, AppError>;
    async fn create_task(
        &self,
        req: CreateTaskRequest,
        actor: &str,
    ) -> Result<TaskResponse, AppError>;
    async fn update_task(
        &self,
        id: &str,
        req: UpdateTaskRequest,
        actor: &str,
    ) -> Result<TaskResponse, AppError>;
    async fn update_task_status(
        &self,
        id: &str,
        req: UpdateTaskStatusRequest,
        actor: &str,
    ) -> Result<TaskResponse, AppError>;
    async fn delete_task(&self, id: &str) -> Result<(), AppError>;
}

pub struct TaskServiceImpl {
    repo: Arc<dyn TaskRepository>,
}

impl TaskServiceImpl {
    pub fn new(repo: Arc<dyn TaskRepository>) -> Self {
        Self { repo }
    }

    fn row_to_response(&self, row: TaskJoinedRow, today: NaiveDate) -> TaskResponse {
        let status = TaskStatus::from_str(&row.status).unwrap_or(TaskStatus::Open);
        let priority = TaskPriority::from_str(&row.priority).unwrap_or(TaskPriority::Medium);
        let is_overdue = is_task_overdue(status, row.due_date, today);

        TaskResponse {
            id: row.id,
            title: row.title,
            description: row.description,
            status,
            priority,
            story_points: row.story_points,
            due_date: row.due_date,
            is_overdue,
            assigned_to: row.assigned_to,
            assigned_username: row.assigned_username,
            assigned_name: row.assigned_name,
            session_id: row.session_id,
            session_name: row.session_name,
            sop_id: row.sop_id,
            sop_title: row.sop_title,
            completed_at: row.completed_at,
            created_at: row.created_at,
            created_by: row.created_by,
            modify_at: row.modify_at,
            modify_by: row.modify_by,
        }
    }
}

#[async_trait]
impl TaskService for TaskServiceImpl {
    async fn get_tasks(&self, query: &TaskQuery) -> Result<TaskListResponse, AppError> {
        let today = query.date.unwrap_or_else(today_wib);
        let rows = self.repo.find_all(query).await?;

        let overdue_only = query.overdue_only.unwrap_or(false);
        let mut kpi_tuples = Vec::with_capacity(rows.len());
        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let item = self.row_to_response(row, today);
            if overdue_only && !item.is_overdue {
                continue;
            }
            kpi_tuples.push((item.status, item.due_date, item.story_points));
            items.push(item);
        }

        let kpi = calculate_task_kpi(&kpi_tuples, today);

        Ok(TaskListResponse { kpi, items })
    }

    async fn get_task_by_id(
        &self,
        id: &str,
        ref_date: Option<NaiveDate>,
    ) -> Result<TaskResponse, AppError> {
        let today = ref_date.unwrap_or_else(today_wib);
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with ID '{id}' not found")))?;

        Ok(self.row_to_response(row, today))
    }

    async fn create_task(
        &self,
        req: CreateTaskRequest,
        actor: &str,
    ) -> Result<TaskResponse, AppError> {
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::BadRequest(
                "Judul tugas tidak boleh kosong".to_string(),
            ));
        }

        if title.chars().count() > 150 {
            return Err(AppError::BadRequest(
                "Judul tugas maksimal 150 karakter".to_string(),
            ));
        }

        let description = req
            .description
            .as_deref()
            .map(|d| d.trim().to_string())
            .unwrap_or_default();

        if description.chars().count() > 2000 {
            return Err(AppError::BadRequest(
                "Deskripsi tugas maksimal 2000 karakter".to_string(),
            ));
        }

        let story_points = req.story_points.unwrap_or(0);
        if !(0..=100).contains(&story_points) {
            return Err(AppError::BadRequest(
                "Story points harus bernilai antara 0 dan 100".to_string(),
            ));
        }

        let assigned_to = match req.assigned_to.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => {
                if !self.repo.check_user_exists(s).await? {
                    return Err(AppError::BadRequest(format!(
                        "Pengguna '{s}' tidak ditemukan"
                    )));
                }
                Some(s.to_string())
            }
            _ => None,
        };

        let session_id = match req.session_id.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => {
                if !self.repo.check_session_exists(s).await? {
                    return Err(AppError::BadRequest(format!(
                        "Sesi tanam '{s}' tidak ditemukan"
                    )));
                }
                Some(s.to_string())
            }
            _ => None,
        };

        let sop_id = match req.sop_id.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => {
                if !self.repo.check_sop_exists(s).await? {
                    return Err(AppError::BadRequest(format!(
                        "Dokumen SOP '{s}' tidak ditemukan"
                    )));
                }
                Some(s.to_string())
            }
            _ => None,
        };

        let status = req.status.unwrap_or(TaskStatus::Open);
        let priority = req.priority.unwrap_or(TaskPriority::Medium);
        let now = Utc::now();
        let completed_at = if status == TaskStatus::Complete {
            Some(now)
        } else {
            None
        };

        let task_id = format!("tsk_{}", Uuid::new_v4());

        let row = TaskJoinedRow {
            id: task_id,
            title,
            description,
            status: status.to_string(),
            priority: priority.to_string(),
            story_points,
            due_date: req.due_date,
            assigned_to,
            assigned_username: None,
            assigned_name: None,
            session_id,
            session_name: None,
            sop_id,
            sop_title: None,
            completed_at,
            created_at: now,
            created_by: actor.to_string(),
            modify_at: now,
            modify_by: actor.to_string(),
        };

        let saved = self.repo.create(row).await?;
        Ok(self.row_to_response(saved, today_wib()))
    }

    async fn update_task(
        &self,
        id: &str,
        req: UpdateTaskRequest,
        actor: &str,
    ) -> Result<TaskResponse, AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with ID '{id}' not found")))?;

        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::BadRequest(
                "Judul tugas tidak boleh kosong".to_string(),
            ));
        }

        if title.chars().count() > 150 {
            return Err(AppError::BadRequest(
                "Judul tugas maksimal 150 karakter".to_string(),
            ));
        }

        let description = req
            .description
            .as_deref()
            .map(|d| d.trim().to_string())
            .unwrap_or_default();

        if description.chars().count() > 2000 {
            return Err(AppError::BadRequest(
                "Deskripsi tugas maksimal 2000 karakter".to_string(),
            ));
        }

        let story_points = req.story_points.unwrap_or(existing.story_points);
        if !(0..=100).contains(&story_points) {
            return Err(AppError::BadRequest(
                "Story points harus bernilai antara 0 dan 100".to_string(),
            ));
        }

        let assigned_to = match req.assigned_to.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => {
                if !self.repo.check_user_exists(s).await? {
                    return Err(AppError::BadRequest(format!(
                        "Pengguna '{s}' tidak ditemukan"
                    )));
                }
                Some(s.to_string())
            }
            _ => None,
        };

        let session_id = match req.session_id.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => {
                if !self.repo.check_session_exists(s).await? {
                    return Err(AppError::BadRequest(format!(
                        "Sesi tanam '{s}' tidak ditemukan"
                    )));
                }
                Some(s.to_string())
            }
            _ => None,
        };

        let sop_id = match req.sop_id.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => {
                if !self.repo.check_sop_exists(s).await? {
                    return Err(AppError::BadRequest(format!(
                        "Dokumen SOP '{s}' tidak ditemukan"
                    )));
                }
                Some(s.to_string())
            }
            _ => None,
        };

        let status = req
            .status
            .unwrap_or_else(|| TaskStatus::from_str(&existing.status).unwrap_or(TaskStatus::Open));
        let priority = req.priority.unwrap_or_else(|| {
            TaskPriority::from_str(&existing.priority).unwrap_or(TaskPriority::Medium)
        });

        let completed_at = if status == TaskStatus::Complete {
            if existing.completed_at.is_some() {
                existing.completed_at
            } else {
                Some(Utc::now())
            }
        } else {
            None
        };

        let updated = self
            .repo
            .update(
                id,
                &title,
                &description,
                &status.to_string(),
                &priority.to_string(),
                story_points,
                req.due_date,
                assigned_to.as_deref(),
                session_id.as_deref(),
                sop_id.as_deref(),
                completed_at,
                actor,
            )
            .await?;

        Ok(self.row_to_response(updated, today_wib()))
    }

    async fn update_task_status(
        &self,
        id: &str,
        req: UpdateTaskStatusRequest,
        actor: &str,
    ) -> Result<TaskResponse, AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with ID '{id}' not found")))?;

        if let Some(sp) = req.story_points {
            if !(0..=100).contains(&sp) {
                return Err(AppError::BadRequest(
                    "Story points harus bernilai antara 0 dan 100".to_string(),
                ));
            }
        }

        let completed_at = if req.status == TaskStatus::Complete {
            if existing.completed_at.is_some() {
                existing.completed_at
            } else {
                Some(Utc::now())
            }
        } else {
            None
        };

        let updated = self
            .repo
            .update_status(
                id,
                &req.status.to_string(),
                req.story_points,
                completed_at,
                actor,
            )
            .await?;

        Ok(self.row_to_response(updated, today_wib()))
    }

    async fn delete_task(&self, id: &str) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("Task with ID '{id}' not found")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simulasi 2: Evaluasi Overdue, Completion Rate, & Story Point Velocity (PRD §3.2 & §3.3)
    #[test]
    fn test_task_simulasi_2_kpi_and_overdue() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap();

        // 1. Tugas A: Complete, due 2026-11-20, SP 5 -> is_overdue = false
        let is_overdue_a = is_task_overdue(
            TaskStatus::Complete,
            Some(NaiveDate::from_ymd_opt(2026, 11, 20).unwrap()),
            today,
        );
        assert!(!is_overdue_a);

        // 2. Tugas B: In Progress, due 2026-11-22, SP 8 -> is_overdue = true
        let is_overdue_b = is_task_overdue(
            TaskStatus::InProgress,
            Some(NaiveDate::from_ymd_opt(2026, 11, 22).unwrap()),
            today,
        );
        assert!(is_overdue_b);

        // 3. Tugas C: In Review, due 2026-11-24, SP 3 -> is_overdue = false
        let is_overdue_c = is_task_overdue(
            TaskStatus::InReview,
            Some(NaiveDate::from_ymd_opt(2026, 11, 24).unwrap()),
            today,
        );
        assert!(!is_overdue_c);

        // 4. Tugas D: Complete, due 2026-11-25, SP 4 -> is_overdue = false
        let is_overdue_d = is_task_overdue(
            TaskStatus::Complete,
            Some(NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()),
            today,
        );
        assert!(!is_overdue_d);

        // KPI Summary Calculation
        let tasks = vec![
            (
                TaskStatus::Complete,
                Some(NaiveDate::from_ymd_opt(2026, 11, 20).unwrap()),
                5,
            ),
            (
                TaskStatus::InProgress,
                Some(NaiveDate::from_ymd_opt(2026, 11, 22).unwrap()),
                8,
            ),
            (
                TaskStatus::InReview,
                Some(NaiveDate::from_ymd_opt(2026, 11, 24).unwrap()),
                3,
            ),
            (
                TaskStatus::Complete,
                Some(NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()),
                4,
            ),
        ];

        let kpi = calculate_task_kpi(&tasks, today);
        assert_eq!(kpi.total_tasks, 4);
        assert_eq!(kpi.completed_tasks, 2);
        assert_eq!(kpi.overdue_tasks, 1);
        assert_eq!(kpi.completion_rate_percent, Decimal::new(5000, 2)); // 50.00%
        assert_eq!(kpi.total_story_points, 20);
        assert_eq!(kpi.completed_story_points, 9);
        assert_eq!(kpi.story_point_velocity_percent, Decimal::new(4500, 2)); // 45.00%
    }
}
