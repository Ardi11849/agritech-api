use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum SopCategory {
    Nutrisi,
    Hama,
    Panen,
    Maintenance,
    Pembibitan,
    Umum,
}

impl fmt::Display for SopCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nutrisi => write!(f, "Nutrisi"),
            Self::Hama => write!(f, "Hama"),
            Self::Panen => write!(f, "Panen"),
            Self::Maintenance => write!(f, "Maintenance"),
            Self::Pembibitan => write!(f, "Pembibitan"),
            Self::Umum => write!(f, "Umum"),
        }
    }
}

impl FromStr for SopCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Nutrisi" => Ok(Self::Nutrisi),
            "Hama" => Ok(Self::Hama),
            "Panen" => Ok(Self::Panen),
            "Maintenance" => Ok(Self::Maintenance),
            "Pembibitan" => Ok(Self::Pembibitan),
            "Umum" => Ok(Self::Umum),
            other => Err(format!("Unknown SOP category: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum SopStatus {
    Draft,
    Published,
    Archived,
}

impl fmt::Display for SopStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Published => write!(f, "Published"),
            Self::Archived => write!(f, "Archived"),
        }
    }
}

impl FromStr for SopStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Draft" => Ok(Self::Draft),
            "Published" => Ok(Self::Published),
            "Archived" => Ok(Self::Archived),
            other => Err(format!("Unknown SOP status: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SopStepInput {
    pub instruction: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSopStep {
    pub step_number: i32,
    pub instruction: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SopStepResponse {
    pub id: String,
    pub sop_id: String,
    pub step_number: i32,
    pub instruction: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SopResponse {
    pub id: String,
    pub title: String,
    pub category: SopCategory,
    pub description: String,
    pub version: i32,
    pub status: SopStatus,
    pub step_count: i32,
    pub steps: Vec<SopStepResponse>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SopKpi {
    pub total_sops: i64,
    pub published_sops: i64,
    pub draft_sops: i64,
    pub archived_sops: i64,
    pub total_steps: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SopListResponse {
    pub kpi: SopKpi,
    pub items: Vec<SopResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSopRequest {
    pub title: String,
    pub category: SopCategory,
    pub description: Option<String>,
    pub status: Option<SopStatus>,
    pub steps: Vec<SopStepInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSopRequest {
    pub title: String,
    pub category: SopCategory,
    pub description: Option<String>,
    pub status: Option<SopStatus>,
    pub steps: Vec<SopStepInput>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct SopQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SopRow {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub version: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub modify_at: DateTime<Utc>,
    pub modify_by: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SopStepRow {
    pub id: String,
    pub sop_id: String,
    pub step_number: i32,
    pub instruction: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}
