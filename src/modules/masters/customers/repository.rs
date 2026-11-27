use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::customers::dto::{CustomerEntity, CustomerListQuery};

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn find_all(&self, query: &CustomerListQuery) -> Result<Vec<CustomerEntity>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<CustomerEntity>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<CustomerEntity>, AppError>;
    async fn create(&self, entity: CustomerEntity) -> Result<CustomerEntity, AppError>;
    async fn update(&self, entity: CustomerEntity) -> Result<CustomerEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<bool, AppError>;
}

pub struct PgCustomerRepository {
    pool: PgPool,
}

impl PgCustomerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepository for PgCustomerRepository {
    async fn find_all(&self, query: &CustomerListQuery) -> Result<Vec<CustomerEntity>, AppError> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT id, name, category, contact_person, phone, address, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM customers
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
            .build_query_as::<CustomerEntity>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<CustomerEntity>, AppError> {
        let row = sqlx::query_as::<_, CustomerEntity>(
            r#"
            SELECT id, name, category, contact_person, phone, address, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM customers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<CustomerEntity>, AppError> {
        let row = sqlx::query_as::<_, CustomerEntity>(
            r#"
            SELECT id, name, category, contact_person, phone, address, is_active,
                   created_at, created_by, modify_at, modify_by
            FROM customers
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn create(&self, entity: CustomerEntity) -> Result<CustomerEntity, AppError> {
        let row = sqlx::query_as::<_, CustomerEntity>(
            r#"
            INSERT INTO customers (id, name, category, contact_person, phone, address, is_active, created_at, created_by, modify_at, modify_by)
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

    async fn update(&self, entity: CustomerEntity) -> Result<CustomerEntity, AppError> {
        let row = sqlx::query_as::<_, CustomerEntity>(
            r#"
            UPDATE customers
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
        .ok_or_else(|| AppError::NotFound(format!("Customer with ID '{}' not found", entity.id)))?;

        Ok(row)
    }

    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let res = sqlx::query("DELETE FROM customers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }
}
