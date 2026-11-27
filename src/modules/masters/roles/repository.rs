use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::role::dto::{RoleEntity, RolePermissionEntity};
use crate::modules::system_modules::dto::{GroupEntity, ModuleEntity};

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<RoleEntity>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<RoleEntity>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<RoleEntity>, AppError>;
    async fn create(&self, role: &RoleEntity) -> Result<RoleEntity, AppError>;
    async fn update(&self, role: &RoleEntity) -> Result<RoleEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
    async fn count_users_by_role_id(&self, role_id: &str) -> Result<i64, AppError>;

    async fn find_groups_and_modules_for_role(
        &self,
        role_id: &str,
    ) -> Result<
        (
            Vec<GroupEntity>,
            Vec<ModuleEntity>,
            Vec<RolePermissionEntity>,
        ),
        AppError,
    >;
}

#[async_trait]
pub trait RolePermissionRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<RolePermissionEntity>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<RolePermissionEntity>, AppError>;
    async fn find_by_role_id(&self, role_id: &str) -> Result<Vec<RolePermissionEntity>, AppError>;
    async fn find_module_group_id(&self, module_id: &str) -> Result<Option<String>, AppError>;
    async fn create(&self, perm: &RolePermissionEntity) -> Result<RolePermissionEntity, AppError>;
    async fn update(&self, perm: &RolePermissionEntity) -> Result<RolePermissionEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    async fn find_all(&self) -> Result<Vec<RoleEntity>, AppError> {
        let roles = sqlx::query_as::<_, RoleEntity>(
            r#"
            SELECT id, name, is_active, created_at, created_by, modify_at, modify_by
            FROM roles
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<RoleEntity>, AppError> {
        let role = sqlx::query_as::<_, RoleEntity>(
            r#"
            SELECT id, name, is_active, created_at, created_by, modify_at, modify_by
            FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<RoleEntity>, AppError> {
        let role = sqlx::query_as::<_, RoleEntity>(
            r#"
            SELECT id, name, is_active, created_at, created_by, modify_at, modify_by
            FROM roles
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    async fn create(&self, role: &RoleEntity) -> Result<RoleEntity, AppError> {
        let created = sqlx::query_as::<_, RoleEntity>(
            r#"
            INSERT INTO roles (id, name, is_active, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&role.id)
        .bind(&role.name)
        .bind(role.is_active)
        .bind(role.created_at)
        .bind(&role.created_by)
        .bind(role.modify_at)
        .bind(&role.modify_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn update(&self, role: &RoleEntity) -> Result<RoleEntity, AppError> {
        let updated = sqlx::query_as::<_, RoleEntity>(
            r#"
            UPDATE roles
            SET name = $1, is_active = $2, modify_at = $3, modify_by = $4
            WHERE id = $5
            RETURNING id, name, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&role.name)
        .bind(role.is_active)
        .bind(role.modify_at)
        .bind(&role.modify_by)
        .bind(&role.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Role with ID '{id}' not found")));
        }
        Ok(())
    }

    async fn count_users_by_role_id(&self, role_id: &str) -> Result<i64, AppError> {
        let res: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(res.0)
    }

    async fn find_groups_and_modules_for_role(
        &self,
        role_id: &str,
    ) -> Result<
        (
            Vec<GroupEntity>,
            Vec<ModuleEntity>,
            Vec<RolePermissionEntity>,
        ),
        AppError,
    > {
        let groups = sqlx::query_as::<_, GroupEntity>(
            r#"
            SELECT id, name, icon, is_active, created_at, created_by, modify_at, modify_by
            FROM system_module_groups
            WHERE is_active = true
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let modules = sqlx::query_as::<_, ModuleEntity>(
            r#"
            SELECT id, group_id, parent_id, name, path, icon, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM system_modules
            WHERE is_active = true
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let permissions = sqlx::query_as::<_, RolePermissionEntity>(
            r#"
            SELECT id, role_id, module_group_id, module_id, can_create, can_read,
                   can_update, can_delete, created_at, created_by, modify_at, modify_by
            FROM role_module_permissions
            WHERE role_id = $1
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        Ok((groups, modules, permissions))
    }
}

pub struct PgRolePermissionRepository {
    pool: PgPool,
}

impl PgRolePermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RolePermissionRepository for PgRolePermissionRepository {
    async fn find_all(&self) -> Result<Vec<RolePermissionEntity>, AppError> {
        let perms = sqlx::query_as::<_, RolePermissionEntity>(
            r#"
            SELECT id, role_id, module_group_id, module_id, can_create, can_read,
                   can_update, can_delete, created_at, created_by, modify_at, modify_by
            FROM role_module_permissions
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(perms)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<RolePermissionEntity>, AppError> {
        let perm = sqlx::query_as::<_, RolePermissionEntity>(
            r#"
            SELECT id, role_id, module_group_id, module_id, can_create, can_read,
                   can_update, can_delete, created_at, created_by, modify_at, modify_by
            FROM role_module_permissions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(perm)
    }

    async fn find_by_role_id(&self, role_id: &str) -> Result<Vec<RolePermissionEntity>, AppError> {
        let perms = sqlx::query_as::<_, RolePermissionEntity>(
            r#"
            SELECT id, role_id, module_group_id, module_id, can_create, can_read,
                   can_update, can_delete, created_at, created_by, modify_at, modify_by
            FROM role_module_permissions
            WHERE role_id = $1
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(perms)
    }

    async fn find_module_group_id(&self, module_id: &str) -> Result<Option<String>, AppError> {
        let res: Option<(String,)> =
            sqlx::query_as("SELECT group_id FROM system_modules WHERE id = $1")
                .bind(module_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(res.map(|r| r.0))
    }

    async fn create(&self, perm: &RolePermissionEntity) -> Result<RolePermissionEntity, AppError> {
        let created = sqlx::query_as::<_, RolePermissionEntity>(
            r#"
            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id, 
                can_create, can_read, can_update, can_delete, 
                created_at, created_by, modify_at, modify_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (role_id, module_group_id, module_id)
            DO UPDATE SET
                can_create = EXCLUDED.can_create,
                can_read = EXCLUDED.can_read,
                can_update = EXCLUDED.can_update,
                can_delete = EXCLUDED.can_delete,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = EXCLUDED.modify_by
            RETURNING id, role_id, module_group_id, module_id, can_create, can_read, can_update, can_delete, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&perm.id)
        .bind(&perm.role_id)
        .bind(&perm.module_group_id)
        .bind(&perm.module_id)
        .bind(perm.can_create)
        .bind(perm.can_read)
        .bind(perm.can_update)
        .bind(perm.can_delete)
        .bind(perm.created_at)
        .bind(&perm.created_by)
        .bind(perm.modify_at)
        .bind(&perm.modify_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn update(&self, perm: &RolePermissionEntity) -> Result<RolePermissionEntity, AppError> {
        let updated = sqlx::query_as::<_, RolePermissionEntity>(
            r#"
            UPDATE role_module_permissions
            SET can_create = $1, can_read = $2, can_update = $3, can_delete = $4,
                modify_at = $5, modify_by = $6
            WHERE id = $7
            RETURNING id, role_id, module_group_id, module_id, can_create, can_read, can_update, can_delete, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(perm.can_create)
        .bind(perm.can_read)
        .bind(perm.can_update)
        .bind(perm.can_delete)
        .bind(perm.modify_at)
        .bind(&perm.modify_by)
        .bind(&perm.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM role_module_permissions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Role permission with ID '{id}' not found"
            )));
        }
        Ok(())
    }
}
