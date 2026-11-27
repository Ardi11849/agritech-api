use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::auth::dto::UserEntity;

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<UserEntity>, AppError>;
    async fn find_by_username_or_email(
        &self,
        username: &str,
        email: &str,
    ) -> Result<Option<UserEntity>, AppError>;
    async fn role_exists(&self, role_id: &str) -> Result<bool, AppError>;
    async fn create_user(&self, user: &UserEntity) -> Result<UserEntity, AppError>;
}

pub struct PgAuthRepository {
    pool: PgPool,
}

impl PgAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRepository for PgAuthRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<UserEntity>, AppError> {
        let user = sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, name, email, username, password, role_id, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_username_or_email(
        &self,
        username: &str,
        email: &str,
    ) -> Result<Option<UserEntity>, AppError> {
        let user = sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, name, email, username, password, role_id, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM users
            WHERE username = $1 OR email = $2
            "#,
        )
        .bind(username)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn role_exists(&self, role_id: &str) -> Result<bool, AppError> {
        let exists: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(exists.is_some())
    }

    async fn create_user(&self, user: &UserEntity) -> Result<UserEntity, AppError> {
        let created = sqlx::query_as::<_, UserEntity>(
            r#"
            INSERT INTO users (id, name, email, username, password, role_id, is_active, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, email, username, password, role_id, is_active,
                      created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.role_id)
        .bind(user.is_active)
        .bind(user.created_at)
        .bind(&user.created_by)
        .bind(user.modify_at)
        .bind(&user.modify_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }
}
