use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::item_categories::dto::{
    ItemCategoryEntity, ItemCategoryListQuery, ItemCategorySummaryRow,
};

#[async_trait]
pub trait ItemCategoryRepository: Send + Sync {
    async fn find_all(
        &self,
        query: &ItemCategoryListQuery,
    ) -> Result<Vec<ItemCategorySummaryRow>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<ItemCategorySummaryRow>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<ItemCategoryEntity>, AppError>;
    async fn count_items_by_category(&self, category_id: &str) -> Result<i64, AppError>;
    async fn create(&self, cat: &ItemCategoryEntity) -> Result<ItemCategoryEntity, AppError>;
    async fn update(&self, cat: &ItemCategoryEntity) -> Result<ItemCategoryEntity, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgItemCategoryRepository {
    pool: PgPool,
}

impl PgItemCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ItemCategoryRepository for PgItemCategoryRepository {
    async fn find_all(
        &self,
        query: &ItemCategoryListQuery,
    ) -> Result<Vec<ItemCategorySummaryRow>, AppError> {
        let mut sql = String::from(
            r#"
            SELECT
                c.id, c.name, c.description, c.is_active,
                c.created_at, c.created_by, c.modify_at, c.modify_by,
                COUNT(i.id)::BIGINT AS item_count,
                COUNT(i.id) FILTER (WHERE i.is_active = TRUE)::BIGINT AS active_item_count
            FROM item_categories c
            LEFT JOIN master_items i ON i.category_id = c.id
            WHERE 1=1
            "#,
        );

        if query.is_active.is_some() {
            sql.push_str(" AND c.is_active = $1");
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                if query.is_active.is_some() {
                    sql.push_str(" AND (c.name ILIKE $2 OR c.description ILIKE $2)");
                } else {
                    sql.push_str(" AND (c.name ILIKE $1 OR c.description ILIKE $1)");
                }
            }
        }

        sql.push_str(" GROUP BY c.id ORDER BY c.name ASC");

        let mut q = sqlx::query_as::<_, ItemCategorySummaryRow>(&sql);

        if let Some(active) = query.is_active {
            q = q.bind(active);
        }

        if let Some(ref s) = query.search {
            if !s.trim().is_empty() {
                q = q.bind(format!("%{}%", s.trim()));
            }
        }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<ItemCategorySummaryRow>, AppError> {
        let row = sqlx::query_as::<_, ItemCategorySummaryRow>(
            r#"
            SELECT
                c.id, c.name, c.description, c.is_active,
                c.created_at, c.created_by, c.modify_at, c.modify_by,
                COUNT(i.id)::BIGINT AS item_count,
                COUNT(i.id) FILTER (WHERE i.is_active = TRUE)::BIGINT AS active_item_count
            FROM item_categories c
            LEFT JOIN master_items i ON i.category_id = c.id
            WHERE c.id = $1
            GROUP BY c.id
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<ItemCategoryEntity>, AppError> {
        let row = sqlx::query_as::<_, ItemCategoryEntity>(
            r#"
            SELECT id, name, description, is_active, created_at, created_by, modify_at, modify_by
            FROM item_categories
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn count_items_by_category(&self, category_id: &str) -> Result<i64, AppError> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM master_items WHERE category_id = $1")
                .bind(category_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    async fn create(&self, cat: &ItemCategoryEntity) -> Result<ItemCategoryEntity, AppError> {
        sqlx::query(
            r#"
            INSERT INTO item_categories (
                id, name, description, is_active,
                created_at, created_by, modify_at, modify_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&cat.id)
        .bind(&cat.name)
        .bind(&cat.description)
        .bind(cat.is_active)
        .bind(cat.created_at)
        .bind(&cat.created_by)
        .bind(cat.modify_at)
        .bind(&cat.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(cat.clone())
    }

    async fn update(&self, cat: &ItemCategoryEntity) -> Result<ItemCategoryEntity, AppError> {
        sqlx::query(
            r#"
            UPDATE item_categories SET
                name = $2,
                description = $3,
                is_active = $4,
                modify_at = $5,
                modify_by = $6
            WHERE id = $1
            "#,
        )
        .bind(&cat.id)
        .bind(&cat.name)
        .bind(&cat.description)
        .bind(cat.is_active)
        .bind(cat.modify_at)
        .bind(&cat.modify_by)
        .execute(&self.pool)
        .await?;

        Ok(cat.clone())
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM item_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
