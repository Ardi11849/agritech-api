use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::auth::dto::UserEntity;
use crate::modules::user::dto::UserItemResponse;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<UserEntity>, AppError>;
    async fn find_all_users(&self) -> Result<Vec<UserItemResponse>, AppError>;
    async fn find_user_item_by_id(&self, id: &str) -> Result<Option<UserItemResponse>, AppError>;
    async fn find_by_username_or_email_excluding(
        &self,
        username: &str,
        email: &str,
        exclude_id: Option<&str>,
    ) -> Result<Option<UserEntity>, AppError>;
    async fn create_user(&self, user: &UserEntity) -> Result<UserEntity, AppError>;
    async fn update_user(&self, user: &UserEntity) -> Result<UserEntity, AppError>;
    async fn delete_user(&self, id: &str) -> Result<(), AppError>;
    async fn role_exists(&self, role_id: &str) -> Result<bool, AppError>;
    async fn get_role_name(&self, role_id: &str) -> Result<Option<String>, AppError>;
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<UserEntity>, AppError> {
        let user = sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, name, email, username, password, role_id, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_all_users(&self) -> Result<Vec<UserItemResponse>, AppError> {
        let users = sqlx::query_as::<_, UserItemResponse>(
            r#"
            SELECT u.id, u.name, u.email, u.username, u.role_id, r.name as role_name,
                   u.is_active, u.created_at, u.created_by, u.modify_at, u.modify_by
            FROM users u
            JOIN roles r ON u.role_id = r.id
            ORDER BY u.created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn find_user_item_by_id(&self, id: &str) -> Result<Option<UserItemResponse>, AppError> {
        let user = sqlx::query_as::<_, UserItemResponse>(
            r#"
            SELECT u.id, u.name, u.email, u.username, u.role_id, r.name as role_name,
                   u.is_active, u.created_at, u.created_by, u.modify_at, u.modify_by
            FROM users u
            JOIN roles r ON u.role_id = r.id
            WHERE u.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_username_or_email_excluding(
        &self,
        username: &str,
        email: &str,
        exclude_id: Option<&str>,
    ) -> Result<Option<UserEntity>, AppError> {
        let user = sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, name, email, username, password, role_id, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM users
            WHERE (username = $1 OR email = $2)
              AND ($3::text IS NULL OR id != $3)
            LIMIT 1
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(exclude_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create_user(&self, user: &UserEntity) -> Result<UserEntity, AppError> {
        let created = sqlx::query_as::<_, UserEntity>(
            r#"
            INSERT INTO users (id, name, email, username, password, role_id, is_active, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, email, username, password, role_id, is_active, created_at, created_by, modify_at, modify_by
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

    async fn update_user(&self, user: &UserEntity) -> Result<UserEntity, AppError> {
        let updated = sqlx::query_as::<_, UserEntity>(
            r#"
            UPDATE users
            SET name = $1, email = $2, username = $3, password = $4, role_id = $5,
                is_active = $6, modify_at = $7, modify_by = $8
            WHERE id = $9
            RETURNING id, name, email, username, password, role_id, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.role_id)
        .bind(user.is_active)
        .bind(user.modify_at)
        .bind(&user.modify_by)
        .bind(&user.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete_user(&self, id: &str) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("User with ID '{id}' not found")));
        }

        Ok(())
    }

    async fn role_exists(&self, role_id: &str) -> Result<bool, AppError> {
        let res: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(res.0)
    }

    async fn get_role_name(&self, role_id: &str) -> Result<Option<String>, AppError> {
        let res: Option<(String,)> = sqlx::query_as("SELECT name FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(res.map(|r| r.0))
    }
}
