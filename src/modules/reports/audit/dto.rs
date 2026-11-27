use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum AuditAction {
    CREATE,
    UPDATE,
    DELETE,
    LOGIN,
    LOCK,
    ADJUST,
}

impl fmt::Display for AuditAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditAction::CREATE => write!(f, "CREATE"),
            AuditAction::UPDATE => write!(f, "UPDATE"),
            AuditAction::DELETE => write!(f, "DELETE"),
            AuditAction::LOGIN => write!(f, "LOGIN"),
            AuditAction::LOCK => write!(f, "LOCK"),
            AuditAction::ADJUST => write!(f, "ADJUST"),
        }
    }
}

impl FromStr for AuditAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "CREATE" | "INSERT" => Ok(AuditAction::CREATE),
            "UPDATE" => Ok(AuditAction::UPDATE),
            "DELETE" => Ok(AuditAction::DELETE),
            "LOGIN" => Ok(AuditAction::LOGIN),
            "LOCK" => Ok(AuditAction::LOCK),
            "ADJUST" => Ok(AuditAction::ADJUST),
            _ => Err(format!("Aksi audit tidak valid: '{s}'")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct AuditLogQuery {
    pub module: Option<String>,
    pub action: Option<String>,
    pub username: Option<String>,
    pub search: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditLogItem {
    pub id: String,
    pub user_id: Option<String>,
    pub username: String,
    pub action: AuditAction,
    pub module: String,
    pub target_id: String,
    pub description: String,
    pub old_data: Option<Value>,
    pub new_data: Option<Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditLogListResponse {
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub items: Vec<AuditLogItem>,
}

pub struct NewAuditEventRow {
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
}
