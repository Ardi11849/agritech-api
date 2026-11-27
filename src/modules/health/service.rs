use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::health::dto::{ComponentStatus, HealthStatus};
use crate::modules::health::repository::HealthRepository;

#[async_trait]
pub trait HealthService: Send + Sync {
    async fn check_health(&self) -> HealthStatus;
}

pub struct HealthServiceImpl {
    repo: Arc<dyn HealthRepository>,
}

impl HealthServiceImpl {
    pub fn new(repo: Arc<dyn HealthRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl HealthService for HealthServiceImpl {
    async fn check_health(&self) -> HealthStatus {
        let db_status = match self.repo.ping_db().await {
            Ok(_) => ComponentStatus::Ok,
            Err(_) => ComponentStatus::Down,
        };

        HealthStatus {
            app: ComponentStatus::Ok,
            database: db_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::error::AppError;

    struct MockHealthyRepo;
    #[async_trait]
    impl HealthRepository for MockHealthyRepo {
        async fn ping_db(&self) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockUnhealthyRepo;
    #[async_trait]
    impl HealthRepository for MockUnhealthyRepo {
        async fn ping_db(&self) -> Result<(), AppError> {
            Err(AppError::Database("connection refused".to_string()))
        }
    }

    #[tokio::test]
    async fn test_health_check_happy_path() {
        let repo = Arc::new(MockHealthyRepo);
        let service = HealthServiceImpl::new(repo);
        let status = service.check_health().await;

        assert_eq!(status.app, ComponentStatus::Ok);
        assert_eq!(status.database, ComponentStatus::Ok);
    }

    #[tokio::test]
    async fn test_health_check_database_down() {
        let repo = Arc::new(MockUnhealthyRepo);
        let service = HealthServiceImpl::new(repo);
        let status = service.check_health().await;

        assert_eq!(status.app, ComponentStatus::Ok);
        assert_eq!(status.database, ComponentStatus::Down);
    }
}
