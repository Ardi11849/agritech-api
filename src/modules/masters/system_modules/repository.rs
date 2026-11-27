use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::system_modules::dto::{GroupEntity, ModuleEntity};

#[async_trait]
pub trait SystemModulesRepository: Send + Sync {
    async fn find_all_groups(&self) -> Result<Vec<GroupEntity>, AppError>;
    async fn find_group_by_id(&self, id: &str) -> Result<Option<GroupEntity>, AppError>;
    async fn create_group(&self, group: &GroupEntity) -> Result<GroupEntity, AppError>;
    async fn update_group(&self, group: &GroupEntity) -> Result<GroupEntity, AppError>;
    async fn delete_group(&self, id: &str) -> Result<(), AppError>;

    async fn find_all_modules(&self) -> Result<Vec<ModuleEntity>, AppError>;
    async fn find_modules_by_group(&self, group_id: &str) -> Result<Vec<ModuleEntity>, AppError>;
    async fn find_module_by_id(&self, id: &str) -> Result<Option<ModuleEntity>, AppError>;
    async fn create_module(&self, module: &ModuleEntity) -> Result<ModuleEntity, AppError>;
    async fn update_module(&self, module: &ModuleEntity) -> Result<ModuleEntity, AppError>;
    async fn delete_module(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgSystemModulesRepository {
    pool: PgPool,
}

impl PgSystemModulesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SystemModulesRepository for PgSystemModulesRepository {
    async fn find_all_groups(&self) -> Result<Vec<GroupEntity>, AppError> {
        let groups = sqlx::query_as::<_, GroupEntity>(
            r#"
            SELECT id, name, icon, is_active, created_at, created_by, modify_at, modify_by
            FROM system_module_groups
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    async fn find_group_by_id(&self, id: &str) -> Result<Option<GroupEntity>, AppError> {
        let group = sqlx::query_as::<_, GroupEntity>(
            r#"
            SELECT id, name, icon, is_active, created_at, created_by, modify_at, modify_by
            FROM system_module_groups
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(group)
    }

    async fn create_group(&self, group: &GroupEntity) -> Result<GroupEntity, AppError> {
        let created = sqlx::query_as::<_, GroupEntity>(
            r#"
            INSERT INTO system_module_groups (id, name, icon, is_active, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, icon, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&group.id)
        .bind(&group.name)
        .bind(&group.icon)
        .bind(group.is_active)
        .bind(group.created_at)
        .bind(&group.created_by)
        .bind(group.modify_at)
        .bind(&group.modify_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn update_group(&self, group: &GroupEntity) -> Result<GroupEntity, AppError> {
        let updated = sqlx::query_as::<_, GroupEntity>(
            r#"
            UPDATE system_module_groups
            SET name = $1, icon = $2, is_active = $3, modify_at = $4, modify_by = $5
            WHERE id = $6
            RETURNING id, name, icon, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&group.name)
        .bind(&group.icon)
        .bind(group.is_active)
        .bind(group.modify_at)
        .bind(&group.modify_by)
        .bind(&group.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete_group(&self, id: &str) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM system_module_groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Module group with ID '{id}' not found"
            )));
        }
        Ok(())
    }

    async fn find_all_modules(&self) -> Result<Vec<ModuleEntity>, AppError> {
        let modules = sqlx::query_as::<_, ModuleEntity>(
            r#"
            SELECT id, group_id, parent_id, name, path, icon, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM system_modules
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(modules)
    }

    async fn find_modules_by_group(&self, group_id: &str) -> Result<Vec<ModuleEntity>, AppError> {
        let modules = sqlx::query_as::<_, ModuleEntity>(
            r#"
            SELECT id, group_id, parent_id, name, path, icon, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM system_modules
            WHERE group_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(modules)
    }

    async fn find_module_by_id(&self, id: &str) -> Result<Option<ModuleEntity>, AppError> {
        let module = sqlx::query_as::<_, ModuleEntity>(
            r#"
            SELECT id, group_id, parent_id, name, path, icon, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM system_modules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(module)
    }

    async fn create_module(&self, module: &ModuleEntity) -> Result<ModuleEntity, AppError> {
        let created = sqlx::query_as::<_, ModuleEntity>(
            r#"
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active,
                                       created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, group_id, parent_id, name, path, icon, is_active,
                      created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&module.id)
        .bind(&module.group_id)
        .bind(&module.parent_id)
        .bind(&module.name)
        .bind(&module.path)
        .bind(&module.icon)
        .bind(module.is_active)
        .bind(module.created_at)
        .bind(&module.created_by)
        .bind(module.modify_at)
        .bind(&module.modify_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn update_module(&self, module: &ModuleEntity) -> Result<ModuleEntity, AppError> {
        let updated = sqlx::query_as::<_, ModuleEntity>(
            r#"
            UPDATE system_modules
            SET group_id = $1, parent_id = $2, name = $3, path = $4, icon = $5,
                is_active = $6, modify_at = $7, modify_by = $8
            WHERE id = $9
            RETURNING id, group_id, parent_id, name, path, icon, is_active,
                      created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&module.group_id)
        .bind(&module.parent_id)
        .bind(&module.name)
        .bind(&module.path)
        .bind(&module.icon)
        .bind(module.is_active)
        .bind(module.modify_at)
        .bind(&module.modify_by)
        .bind(&module.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete_module(&self, id: &str) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM system_modules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Module with ID '{id}' not found"
            )));
        }
        Ok(())
    }
}
