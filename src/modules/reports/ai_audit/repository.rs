use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PurchaseItemHistoryRow {
    pub purchase_item_id: String,
    pub purchase_id: String,
    pub invoice_number: String,
    pub transaction_date: NaiveDate,
    pub item_name: String,
    pub unit: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
    pub effective_unit_cost: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SaleAuditRow {
    pub sale_id: String,
    pub sale_number: String,
    pub session_id: String,
    pub sale_date: NaiveDate,
    pub unit: String,
    pub quantity: Decimal,
    pub total_revenue: Decimal,
    pub seller_shipping_subsidy: Decimal,
    pub net_proceeds: Decimal,
    pub status: String,
    pub payment_status: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryAuditItemRow {
    pub item_id: String,
    pub item_name: String,
    pub min_stock_level: Decimal,
    pub current_stock: Decimal,
    pub average_unit_cost: Decimal,
    pub default_unit: String,
    pub item_created_at: DateTime<Utc>,
    pub last_stock_out_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait AiAuditRepository: Send + Sync {
    async fn fetch_purchase_items_history(
        &self,
        cutoff_date: NaiveDate,
    ) -> Result<Vec<PurchaseItemHistoryRow>, AppError>;

    async fn fetch_sales_history(
        &self,
        cutoff_date: NaiveDate,
    ) -> Result<Vec<SaleAuditRow>, AppError>;

    async fn fetch_inventory_audit_items(&self) -> Result<Vec<InventoryAuditItemRow>, AppError>;
}

pub struct PgAiAuditRepository {
    pool: PgPool,
}

impl PgAiAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AiAuditRepository for PgAiAuditRepository {
    async fn fetch_purchase_items_history(
        &self,
        cutoff_date: NaiveDate,
    ) -> Result<Vec<PurchaseItemHistoryRow>, AppError> {
        let rows = sqlx::query_as::<_, PurchaseItemHistoryRow>(
            r#"
            SELECT
                pi.id AS purchase_item_id,
                pi.purchase_id,
                ph.invoice_number,
                ph.transaction_date,
                pi.item_name,
                pi.unit,
                pi.quantity,
                pi.total_cost,
                CASE
                    WHEN pi.quantity > 0 THEN ROUND((pi.total_cost / pi.quantity)::numeric, 2)
                    ELSE pi.price_per_unit
                END AS effective_unit_cost,
                ph.created_at
            FROM purchase_items pi
            JOIN purchase_headers ph ON ph.id = pi.purchase_id
            WHERE ph.transaction_date >= $1
            ORDER BY ph.transaction_date DESC, ph.created_at DESC, pi.line_number ASC
            "#,
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn fetch_sales_history(
        &self,
        cutoff_date: NaiveDate,
    ) -> Result<Vec<SaleAuditRow>, AppError> {
        let rows = sqlx::query_as::<_, SaleAuditRow>(
            r#"
            SELECT
                s.id AS sale_id,
                s.sale_number,
                s.session_id,
                s.sale_date,
                s.unit,
                s.quantity,
                s.total_revenue,
                s.seller_shipping_subsidy,
                s.net_proceeds,
                s.status,
                s.payment_status
            FROM sales s
            WHERE s.sale_date >= $1
            ORDER BY s.sale_date DESC, s.created_at DESC
            "#,
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn fetch_inventory_audit_items(&self) -> Result<Vec<InventoryAuditItemRow>, AppError> {
        let rows = sqlx::query_as::<_, InventoryAuditItemRow>(
            r#"
            SELECT
                mi.id AS item_id,
                mi.name AS item_name,
                mi.min_stock_level,
                mi.current_stock,
                mi.average_unit_cost,
                mi.default_unit,
                mi.created_at AS item_created_at,
                (
                    SELECT MAX(il.transaction_date)
                    FROM inventory_ledgers il
                    WHERE il.item_id = mi.id
                      AND il.movement_type = 'STOCK_OUT'
                ) AS last_stock_out_at
            FROM master_items mi
            WHERE mi.is_active = TRUE
            ORDER BY mi.name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
