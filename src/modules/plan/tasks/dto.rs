use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum TaskStatus {
    #[serde(rename = "Open")]
    Open,
    #[serde(rename = "Set Story Point")]
    SetStoryPoint,
    #[serde(rename = "In Progress")]
    InProgress,
    #[serde(rename = "Resolve")]
    Resolve,
    #[serde(rename = "In Review")]
    InReview,
    #[serde(rename = "Complete")]
    Complete,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::SetStoryPoint => write!(f, "Set Story Point"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Resolve => write!(f, "Resolve"),
            Self::InReview => write!(f, "In Review"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Open" => Ok(Self::Open),
            "Set Story Point" => Ok(Self::SetStoryPoint),
            "In Progress" => Ok(Self::InProgress),
            "Resolve" => Ok(Self::Resolve),
            "In Review" => Ok(Self::InReview),
            "Complete" => Ok(Self::Complete),
            other => Err(format!("Unknown task status: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

impl FromStr for TaskPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Low" => Ok(Self::Low),
            "Medium" => Ok(Self::Medium),
            "High" => Ok(Self::High),
            other => Err(format!("Unknown task priority: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub story_points: i32,
    pub due_date: Option<NaiveDate>,
    pub is_overdue: bool,
    pub assigned_to: Option<String>,
    pub assigned_username: Option<String>,
    pub assigned_name: Option<String>,
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub sop_id: Option<String>,
    pub sop_title: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskKpi {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub overdue_tasks: i64,
    pub completion_rate_percent: Decimal,
    pub total_story_points: i32,
    pub completed_story_points: i32,
    pub story_point_velocity_percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskListResponse {
    pub kpi: TaskKpi,
    pub items: Vec<TaskResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub story_points: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<String>,
    pub session_id: Option<String>,
    pub sop_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub story_points: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<String>,
    pub session_id: Option<String>,
    pub sop_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
    pub story_points: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct TaskQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub session_id: Option<String>,
    pub sop_id: Option<String>,
    pub overdue_only: Option<bool>,
    pub search: Option<String>,
    pub date: Option<NaiveDate>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskJoinedRow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub story_points: i32,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<String>,
    pub assigned_username: Option<String>,
    pub assigned_name: Option<String>,
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub sop_id: Option<String>,
    pub sop_title: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}
