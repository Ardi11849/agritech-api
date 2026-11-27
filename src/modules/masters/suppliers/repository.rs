use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::suppliers::dto::{SupplierEntity, SupplierListQuery};

#[async_trait]
pub trait SupplierRepository: Send + Sync {
    async fn find_all(&self, query: &SupplierListQuery) -> Result<Vec<SupplierEntity>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<SupplierEntity>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<SupplierEntity>, AppError>;
    async fn create(&self, entity: SupplierEntity) -> Result<SupplierEntity, AppError>;
    async fn update(&self, entity: SupplierEntity) -> Result<SupplierEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<bool, AppError>;
}

pub struct PgSupplierRepository {
    pool: PgPool,
}

impl PgSupplierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SupplierRepository for PgSupplierRepository {
    async fn find_all(&self, query: &SupplierListQuery) -> Result<Vec<SupplierEntity>, AppError> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT id, name, category, contact_person, phone, address, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM suppliers
            WHERE 1=1
            "#,
        );

        if let Some(category) = &query.category {
            let trimmed = category.trim();
            if !trimmed.is_empty() {
                qb.push(" AND category ILIKE ");
                qb.push_bind(trimmed);
            }
        }

        if let Some(search) = &query.search {
            let pattern = format!("%{}%", search.trim());
            qb.push(" AND (name ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR contact_person ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR phone ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR address ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(is_active) = query.is_active {
            qb.push(" AND is_active = ");
            qb.push_bind(is_active);
        }

        qb.push(" ORDER BY name ASC");

        let rows = qb
            .build_query_as::<SupplierEntity>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<SupplierEntity>, AppError> {
        let row = sqlx::query_as::<_, SupplierEntity>(
            r#"
            SELECT id, name, category, contact_person, phone, address, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM suppliers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<SupplierEntity>, AppError> {
        let row = sqlx::query_as::<_, SupplierEntity>(
            r#"
            SELECT id, name, category, contact_person, phone, address, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM suppliers
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn create(&self, entity: SupplierEntity) -> Result<SupplierEntity, AppError> {
        let row = sqlx::query_as::<_, SupplierEntity>(
            r#"
            INSERT INTO suppliers (id, name, category, contact_person, phone, address, is_active, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, category, contact_person, phone, address, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.name)
        .bind(&entity.category)
        .bind(&entity.contact_person)
        .bind(&entity.phone)
        .bind(&entity.address)
        .bind(entity.is_active)
        .bind(entity.created_at)
        .bind(&entity.created_by)
        .bind(entity.modify_at)
        .bind(&entity.modify_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, entity: SupplierEntity) -> Result<SupplierEntity, AppError> {
        let row = sqlx::query_as::<_, SupplierEntity>(
            r#"
            UPDATE suppliers
            SET name = $2, category = $3, contact_person = $4, phone = $5, address = $6, is_active = $7, modify_at = $8, modify_by = $9
            WHERE id = $1
            RETURNING id, name, category, contact_person, phone, address, is_active, created_at, created_by, modify_at, modify_by
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.name)
        .bind(&entity.category)
        .bind(&entity.contact_person)
        .bind(&entity.phone)
        .bind(&entity.address)
        .bind(entity.is_active)
        .bind(entity.modify_at)
        .bind(&entity.modify_by)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Supplier with ID '{}' not found", entity.id)))?;

        Ok(row)
    }

    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let res = sqlx::query("DELETE FROM suppliers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }
}
