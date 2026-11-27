use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::modules::reports::audit::dto::{
    AuditAction, AuditLogItem, AuditLogListResponse, AuditLogQuery, NewAuditEventRow,
};
use crate::modules::reports::audit::repository::{
    AuditLogRow, AuditRepository, NormalizedAuditFilter,
};

// Jakarta offset WIB = UTC+7 (7 * 3600 seconds)
const WIB_OFFSET_SECS: i32 = 7 * 3600;

#[async_trait]
pub trait AuditService: Send + Sync {
    async fn get_logs(&self, query: &AuditLogQuery) -> Result<AuditLogListResponse, AppError>;
    async fn get_modules(&self) -> Result<Vec<String>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn record_event(
        &self,
        action: AuditAction,
        module: &str,
        user_id: Option<String>,
        username: &str,
        target_id: &str,
        description: &str,
        old_data: Option<Value>,
        new_data: Option<Value>,
        ip_address: Option<String>,
    ) -> Result<(), AppError>;
}

pub struct AuditServiceImpl {
    repo: Arc<dyn AuditRepository>,
}

impl AuditServiceImpl {
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AuditService for AuditServiceImpl {
    async fn get_logs(&self, query: &AuditLogQuery) -> Result<AuditLogListResponse, AppError> {
        let limit = query.limit.unwrap_or(200);
        if !(1..=500).contains(&limit) {
            return Err(AppError::BadRequest(
                "Parameter 'limit' harus berada di antara 1 dan 500".into(),
            ));
        }

        let offset = query.offset.unwrap_or(0);
        if offset < 0 {
            return Err(AppError::BadRequest(
                "Parameter 'offset' tidak boleh bernilai negatif".into(),
            ));
        }

        if let (Some(start), Some(end)) = (query.start_date, query.end_date) {
            if start > end {
                return Err(AppError::BadRequest(
                    "Tanggal mulai tidak boleh lebih besar dari tanggal akhir".into(),
                ));
            }
        }

        // Action enum parsing with support for INSERT -> CREATE synonym
        let parsed_action = if let Some(a) = &query.action {
            let trimmed = a.trim();
            if !trimmed.is_empty() {
                let act = AuditAction::from_str(trimmed).map_err(AppError::BadRequest)?;
                Some(act.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Multi-module parsing (CSV)
        let mut modules: Vec<String> = Vec::new();
        if let Some(mod_str) = &query.module {
            for m in mod_str.split(',') {
                let trimmed = m.trim().to_uppercase();
                if !trimmed.is_empty() {
                    modules.push(trimmed);
                }
            }
        }

        let wib_tz = chrono::FixedOffset::east_opt(WIB_OFFSET_SECS).unwrap();

        // Convert NaiveDate to UTC DateTime with WIB timezone alignment
        let start_ts: Option<DateTime<Utc>> = query.start_date.and_then(|d| {
            d.and_time(NaiveTime::from_hms_opt(0, 0, 0)?)
                .and_local_timezone(wib_tz)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
        });

        let end_ts_exclusive: Option<DateTime<Utc>> = query.end_date.and_then(|d| {
            let next_day = d.checked_add_signed(Duration::days(1))?;
            next_day
                .and_time(NaiveTime::from_hms_opt(0, 0, 0)?)
                .and_local_timezone(wib_tz)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
        });

        let filter = NormalizedAuditFilter {
            modules,
            action: parsed_action,
            username: query
                .username
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            search: query
                .search
                .as_deref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            start_ts,
            end_ts_exclusive,
            limit,
            offset,
        };

        let (total_count, rows) = self.repo.find_logs(&filter).await?;

        let items: Vec<AuditLogItem> = rows
            .into_iter()
            .map(|r: AuditLogRow| {
                let action = AuditAction::from_str(&r.action).unwrap_or(AuditAction::UPDATE);
                AuditLogItem {
                    id: r.id,
                    user_id: r.user_id,
                    username: r.username,
                    action,
                    module: r.module,
                    target_id: r.target_id,
                    description: r.description,
                    old_data: r.old_data,
                    new_data: r.new_data,
                    ip_address: r.ip_address,
                    created_at: r.created_at,
                }
            })
            .collect();

        Ok(AuditLogListResponse {
            total_count,
            limit,
            offset,
            items,
        })
    }

    async fn get_modules(&self) -> Result<Vec<String>, AppError> {
        self.repo.find_distinct_modules().await
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_event(
        &self,
        action: AuditAction,
        module: &str,
        user_id: Option<String>,
        username: &str,
        target_id: &str,
        description: &str,
        old_data: Option<Value>,
        new_data: Option<Value>,
        ip_address: Option<String>,
    ) -> Result<(), AppError> {
        let input = NewAuditEventRow {
            id: format!("aud_{}", Uuid::new_v4().simple()),
            user_id,
            username: if username.trim().is_empty() {
                "system".to_string()
            } else {
                username.trim().to_string()
            },
            action: action.to_string(),
            module: module.trim().to_uppercase(),
            target_id: target_id.trim().to_string(),
            description: description.trim().to_string(),
            old_data,
            new_data,
            ip_address: ip_address
                .map(|ip| ip.trim().to_string())
                .filter(|ip| !ip.is_empty()),
        };

        self.repo.insert_event(&input).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_audit_action_parsing_and_synonym() {
        assert_eq!(
            AuditAction::from_str("CREATE").unwrap(),
            AuditAction::CREATE
        );
        assert_eq!(
            AuditAction::from_str("create").unwrap(),
            AuditAction::CREATE
        );
        assert_eq!(
            AuditAction::from_str("INSERT").unwrap(),
            AuditAction::CREATE
        );
        assert_eq!(
            AuditAction::from_str("insert").unwrap(),
            AuditAction::CREATE
        );
        assert_eq!(
            AuditAction::from_str("UPDATE").unwrap(),
            AuditAction::UPDATE
        );
        assert_eq!(AuditAction::from_str("LOCK").unwrap(), AuditAction::LOCK);
        assert_eq!(
            AuditAction::from_str("ADJUST").unwrap(),
            AuditAction::ADJUST
        );
        assert_eq!(AuditAction::from_str("LOGIN").unwrap(), AuditAction::LOGIN);
        assert!(AuditAction::from_str("UNKNOWN").is_err());
    }
}
