use sqlx::PgPool;

use crate::common::error::AppError;

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Database(format!("Auto-migration failed: {e}")))?;
    Ok(())
}
