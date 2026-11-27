use async_trait::async_trait;
use sqlx::PgPool;
use std::time::Duration;

use crate::common::error::AppError;

#[async_trait]
pub trait HealthRepository: Send + Sync {
    async fn ping_db(&self) -> Result<(), AppError>;
}

pub struct PgHealthRepository {
    pool: PgPool,
}

impl PgHealthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthRepository for PgHealthRepository {
    async fn ping_db(&self) -> Result<(), AppError> {
        let check_fut = sqlx::query("SELECT 1").execute(&self.pool);
        match tokio::time::timeout(Duration::from_millis(2000), check_fut).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(AppError::Database(e.to_string())),
            Err(_) => Err(AppError::Timeout(
                "Database query ping timed out".to_string(),
            )),
        }
    }
}
