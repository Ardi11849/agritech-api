use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnomalySeverity {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "info")]
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnomalyCategory {
    PURCHASE,
    HPP,
    SALES,
    INVENTORY,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct AiAuditScanQuery {
    pub category: Option<AnomalyCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiAuditStats {
    pub health_score: i64,
    pub critical: i64,
    pub warning: i64,
    pub info: i64,
    pub total_anomalies: i64,
    pub total_impact_rp: Decimal,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiAuditInsight {
    pub id: String,
    pub category: AnomalyCategory,
    #[serde(rename = "type")]
    pub severity: AnomalySeverity,
    pub title: String,
    pub message: String,
    pub suggestion: String,
    pub impact_value: Decimal,
    pub related_id: String,
    pub target_path: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiAuditScanResponse {
    pub stats: AiAuditStats,
    pub insights: Vec<AiAuditInsight>,
}
