use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::common::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SaleCashflowRow {
    pub id: String,
    pub sale_date: NaiveDate,
    pub sale_number: String,
    pub location_id: String,
    pub location_name: String,
    pub session_name: String,
    pub counterparty: String,
    pub quantity: Decimal,
    pub unit: String,
    pub total_revenue: Decimal,
    pub seller_shipping_subsidy: Decimal,
    pub net_proceeds: Decimal,
    pub status: String,
    pub payment_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PurchaseCashflowRow {
    pub id: String,
    pub transaction_date: NaiveDate,
    pub invoice_number: String,
    pub location_id: String,
    pub location_name: String,
    pub counterparty: String,
    pub total_amount: Decimal,
    pub first_item_name: Option<String>,
    pub item_count: i64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait ReportSummaryRepository: Send + Sync {
    async fn fetch_sales_inflows(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        location_id: Option<&str>,
        realized_only: bool,
    ) -> Result<Vec<SaleCashflowRow>, AppError>;

    async fn fetch_purchase_outflows(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        location_id: Option<&str>,
    ) -> Result<Vec<PurchaseCashflowRow>, AppError>;
}

pub struct PgReportSummaryRepository {
    pool: PgPool,
}

impl PgReportSummaryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReportSummaryRepository for PgReportSummaryRepository {
    async fn fetch_sales_inflows(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        location_id: Option<&str>,
        realized_only: bool,
    ) -> Result<Vec<SaleCashflowRow>, AppError> {
        let rows = sqlx::query_as::<_, SaleCashflowRow>(
            r#"
            SELECT
                s.id,
                s.sale_date,
                s.sale_number,
                s.location_id,
                l.name AS location_name,
                ps.session_name,
                COALESCE(NULLIF(TRIM(c.name), ''), NULLIF(TRIM(s.customer_name_snapshot), ''), 'Pelanggan Umum') AS counterparty,
                s.quantity,
                s.unit,
                s.total_revenue,
                s.seller_shipping_subsidy,
                s.net_proceeds,
                s.status,
                s.payment_status,
                s.created_at
            FROM sales s
            JOIN locations l ON l.id = s.location_id
            JOIN planting_sessions ps ON ps.id = s.session_id
            LEFT JOIN customers c ON c.id = s.customer_id
            WHERE s.sale_date >= $1 AND s.sale_date <= $2
              AND ($3::TEXT IS NULL OR s.location_id = $3)
              AND ($4::BOOLEAN = FALSE OR s.status = 'Realized')
            ORDER BY s.sale_date DESC, s.created_at DESC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .bind(location_id)
        .bind(realized_only)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn fetch_purchase_outflows(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        location_id: Option<&str>,
    ) -> Result<Vec<PurchaseCashflowRow>, AppError> {
        let rows = sqlx::query_as::<_, PurchaseCashflowRow>(
            r#"
            SELECT
                p.id,
                p.transaction_date,
                p.invoice_number,
                p.location_id,
                l.name AS location_name,
                COALESCE(NULLIF(TRIM(sup.name), ''), NULLIF(TRIM(p.supplier_name_snapshot), ''), 'Pemasok Umum') AS counterparty,
                p.total_amount,
                (
                    SELECT pi.item_name
                    FROM purchase_items pi
                    WHERE pi.purchase_id = p.id
                    ORDER BY pi.line_number ASC
                    LIMIT 1
                ) AS first_item_name,
                COALESCE((
                    SELECT COUNT(*)::BIGINT
                    FROM purchase_items pi
                    WHERE pi.purchase_id = p.id
                ), 0) AS item_count,
                p.notes,
                p.created_at
            FROM purchase_headers p
            JOIN locations l ON l.id = p.location_id
            LEFT JOIN suppliers sup ON sup.id = p.supplier_id
            WHERE p.transaction_date >= $1 AND p.transaction_date <= $2
              AND ($3::TEXT IS NULL OR p.location_id = $3)
            ORDER BY p.transaction_date DESC, p.created_at DESC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .bind(location_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
