use async_trait::async_trait;
use sqlx::PgPool;

use crate::common::error::AppError;
use crate::modules::masters::items::dto::{ItemListQuery, MasterItemEntity, MasterItemRow};

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn find_all(&self, query: &ItemListQuery) -> Result<Vec<MasterItemRow>, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<MasterItemRow>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<MasterItemEntity>, AppError>;
    async fn category_exists(&self, category_id: &str) -> Result<bool, AppError>;
    async fn count_ledgers_by_item(&self, item_id: &str) -> Result<i64, AppError>;
    async fn create(&self, entity: MasterItemEntity) -> Result<MasterItemRow, AppError>;
    async fn update(&self, entity: MasterItemEntity) -> Result<MasterItemRow, AppError>;
    async fn delete(&self, id: &str) -> Result<bool, AppError>;
}

pub struct PgItemRepository {
    pool: PgPool,
}

impl PgItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ItemRepository for PgItemRepository {
    async fn find_all(&self, query: &ItemListQuery) -> Result<Vec<MasterItemRow>, AppError> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                i.id, i.category_id, c.name AS category_name, i.name, i.default_unit,
                i.min_stock_level, i.current_stock, i.average_unit_cost, i.is_active,
                i.created_at, i.created_by, i.modify_at, i.modify_by
            FROM master_items i
            INNER JOIN item_categories c ON c.id = i.category_id
            WHERE 1=1
            "#,
        );

        if let Some(cat_id) = &query.category_id {
            let trimmed = cat_id.trim();
            if !trimmed.is_empty() {
                qb.push(" AND i.category_id = ");
                qb.push_bind(trimmed);
            }
        }

        if let Some(search) = &query.search {
            let pattern = format!("%{}%", search.trim());
            qb.push(" AND (i.name ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR i.default_unit ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR c.name ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(is_active) = query.is_active {
            qb.push(" AND i.is_active = ");
            qb.push_bind(is_active);
        }

        qb.push(" ORDER BY i.name ASC");

        let rows = qb
            .build_query_as::<MasterItemRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<MasterItemRow>, AppError> {
        let row = sqlx::query_as::<_, MasterItemRow>(
            r#"
            SELECT 
                i.id, i.category_id, c.name AS category_name, i.name, i.default_unit,
                i.min_stock_level, i.current_stock, i.average_unit_cost, i.is_active,
                i.created_at, i.created_by, i.modify_at, i.modify_by
            FROM master_items i
            INNER JOIN item_categories c ON c.id = i.category_id
            WHERE i.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<MasterItemEntity>, AppError> {
        let entity = sqlx::query_as::<_, MasterItemEntity>(
            r#"
            SELECT id, category_id, name, default_unit, min_stock_level, current_stock, average_unit_cost, is_active, created_at, created_by, modify_at, modify_by
            FROM master_items
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn category_exists(&self, category_id: &str) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM item_categories WHERE id = $1)",
        )
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count_ledgers_by_item(&self, item_id: &str) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM inventory_ledgers WHERE item_id = $1",
        )
        .bind(item_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn create(&self, entity: MasterItemEntity) -> Result<MasterItemRow, AppError> {
        sqlx::query(
            r#"
            INSERT INTO master_items (id, category_id, name, default_unit, min_stock_level, current_stock, average_unit_cost, is_active, created_at, created_by, modify_at, modify_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.category_id)
        .bind(&entity.name)
        .bind(&entity.default_unit)
        .bind(entity.min_stock_level)
        .bind(entity.current_stock)
        .bind(entity.average_unit_cost)
        .bind(entity.is_active)
        .bind(entity.created_at)
        .bind(&entity.created_by)
        .bind(entity.modify_at)
        .bind(&entity.modify_by)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&entity.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to load created item".into()))
    }

    async fn update(&self, entity: MasterItemEntity) -> Result<MasterItemRow, AppError> {
        sqlx::query(
            r#"
            UPDATE master_items
            SET category_id = $2, name = $3, default_unit = $4, min_stock_level = $5, current_stock = $6, average_unit_cost = $7, is_active = $8, modify_at = $9, modify_by = $10
            WHERE id = $1
            "#,
        )
        .bind(&entity.id)
        .bind(&entity.category_id)
        .bind(&entity.name)
        .bind(&entity.default_unit)
        .bind(entity.min_stock_level)
        .bind(entity.current_stock)
        .bind(entity.average_unit_cost)
        .bind(entity.is_active)
        .bind(entity.modify_at)
        .bind(&entity.modify_by)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&entity.id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item with ID '{}' not found", entity.id)))
    }

    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let res = sqlx::query("DELETE FROM master_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }
}
