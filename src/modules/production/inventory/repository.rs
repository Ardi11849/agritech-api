use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::modules::inventory::dto::{
    CreateMovementRequest, CreateStockOpnameRequest, InventoryItemSummary, InventoryKpi,
    InventoryLedgerResponse, InventoryLedgerRow, InventorySummaryQuery, InventorySummaryResponse,
    MovementType, StockCardQuery, StockCardResponse,
};
use crate::modules::inventory::service::{
    calculate_item_valuation, calculate_stock_movement, calculate_stock_opname,
};
use crate::modules::masters::items::dto::MasterItemRow;
use crate::modules::masters::items::service::evaluate_stock_health;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn get_summary(
        &self,
        query: &InventorySummaryQuery,
    ) -> Result<InventorySummaryResponse, AppError>;
    async fn find_item_by_id(
        &self,
        item_id: &str,
    ) -> Result<Option<InventoryItemSummary>, AppError>;
    async fn supplier_exists(&self, id: &str) -> Result<bool, AppError>;
    async fn customer_exists(&self, id: &str) -> Result<bool, AppError>;
    async fn installation_exists(&self, id: &str) -> Result<bool, AppError>;
    async fn record_movement(
        &self,
        req: &CreateMovementRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError>;
    async fn record_stock_opname(
        &self,
        req: &CreateStockOpnameRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError>;
    async fn get_stock_card(
        &self,
        item_id: &str,
        query: &StockCardQuery,
    ) -> Result<StockCardResponse, AppError>;
}

pub struct PgInventoryRepository {
    pool: PgPool,
}

impl PgInventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for PgInventoryRepository {
    async fn get_summary(
        &self,
        query: &InventorySummaryQuery,
    ) -> Result<InventorySummaryResponse, AppError> {
        // 1. Calculate KPI from all active items
        let active_items = sqlx::query_as::<_, (Decimal, Decimal, Decimal)>(
            r#"
            SELECT min_stock_level, current_stock, average_unit_cost
            FROM master_items
            WHERE is_active = true
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut total_sku: i64 = 0;
        let mut safe_count: i64 = 0;
        let mut low_stock_count: i64 = 0;
        let mut out_of_stock_count: i64 = 0;
        let mut total_valuation_rp = Decimal::ZERO;

        for (min_stock, current_stock, avg_cost) in active_items {
            total_sku += 1;
            total_valuation_rp += calculate_item_valuation(current_stock, avg_cost);

            match evaluate_stock_health(current_stock, min_stock) {
                crate::modules::masters::items::dto::StockHealthStatus::Safe => safe_count += 1,
                crate::modules::masters::items::dto::StockHealthStatus::LowStock => {
                    low_stock_count += 1
                }
                crate::modules::masters::items::dto::StockHealthStatus::OutOfStock => {
                    out_of_stock_count += 1
                }
            }
        }

        let kpi = InventoryKpi {
            total_sku,
            safe_count,
            low_stock_count,
            out_of_stock_count,
            total_valuation_rp: round_to_two_decimals(total_valuation_rp),
        };

        // 2. Query items with filters
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                i.id, i.category_id, c.name AS category_name, i.name, i.default_unit,
                i.min_stock_level, i.current_stock, i.average_unit_cost, i.is_active,
                i.modify_at
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

        let active_filter = query.is_active.unwrap_or(true);
        qb.push(" AND i.is_active = ");
        qb.push_bind(active_filter);

        qb.push(" ORDER BY i.name ASC");

        let rows = qb
            .build_query_as::<(
                String,
                String,
                String,
                String,
                String,
                Decimal,
                Decimal,
                Decimal,
                bool,
                chrono::DateTime<Utc>,
            )>()
            .fetch_all(&self.pool)
            .await?;

        let mut items = Vec::new();
        for (
            id,
            category_id,
            category_name,
            name,
            default_unit,
            min_stock_level,
            current_stock,
            average_unit_cost,
            is_active,
            modify_at,
        ) in rows
        {
            let stock_health = evaluate_stock_health(current_stock, min_stock_level);
            if let Some(target_health) = query.stock_health {
                if stock_health != target_health {
                    continue;
                }
            }

            let total_valuation_rp = calculate_item_valuation(current_stock, average_unit_cost);
            items.push(InventoryItemSummary {
                id,
                category_id,
                category_name,
                name,
                default_unit,
                min_stock_level,
                current_stock,
                average_unit_cost,
                total_valuation_rp,
                stock_health,
                is_active,
                modify_at,
            });
        }

        Ok(InventorySummaryResponse { kpi, items })
    }

    async fn find_item_by_id(
        &self,
        item_id: &str,
    ) -> Result<Option<InventoryItemSummary>, AppError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                Decimal,
                Decimal,
                Decimal,
                bool,
                chrono::DateTime<Utc>,
            ),
        >(
            r#"
            SELECT 
                i.id, i.category_id, c.name AS category_name, i.name, i.default_unit,
                i.min_stock_level, i.current_stock, i.average_unit_cost, i.is_active,
                i.modify_at
            FROM master_items i
            INNER JOIN item_categories c ON c.id = i.category_id
            WHERE i.id = $1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                id,
                category_id,
                category_name,
                name,
                default_unit,
                min_stock_level,
                current_stock,
                average_unit_cost,
                is_active,
                modify_at,
            )| {
                let stock_health = evaluate_stock_health(current_stock, min_stock_level);
                let total_valuation_rp = calculate_item_valuation(current_stock, average_unit_cost);
                InventoryItemSummary {
                    id,
                    category_id,
                    category_name,
                    name,
                    default_unit,
                    min_stock_level,
                    current_stock,
                    average_unit_cost,
                    total_valuation_rp,
                    stock_health,
                    is_active,
                    modify_at,
                }
            },
        ))
    }

    async fn supplier_exists(&self, id: &str) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM suppliers WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn customer_exists(&self, id: &str) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM customers WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn installation_exists(&self, id: &str) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM installations WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn record_movement(
        &self,
        req: &CreateMovementRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Acquire row lock
        let item = sqlx::query_as::<_, MasterItemRow>(
            r#"
            SELECT 
                i.id, i.category_id, c.name AS category_name, i.name, i.default_unit,
                i.min_stock_level, i.current_stock, i.average_unit_cost, i.is_active,
                i.created_at, i.created_by, i.modify_at, i.modify_by
            FROM master_items i
            INNER JOIN item_categories c ON c.id = i.category_id
            WHERE i.id = $1
            FOR UPDATE OF i
            "#,
        )
        .bind(&req.item_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Item with ID '{}' not found", req.item_id)))?;

        if !item.is_active {
            return Err(AppError::BadRequest(
                "Barang berstatus nonaktif tidak dapat diproses mutasi stoknya".into(),
            ));
        }

        // 2. Pure calculation
        let movement_type = MovementType::from(req.movement_type);
        let calc = calculate_stock_movement(
            item.current_stock,
            item.average_unit_cost,
            movement_type,
            req.quantity,
            req.unit_cost,
        )?;

        // 3. Insert into inventory_ledgers
        let ledger_id = format!("inv_{}", Uuid::new_v4());
        let transaction_date = req.transaction_date.unwrap_or_else(Utc::now);
        let reference_no = req.reference_no.as_deref().unwrap_or("").trim();
        let notes = req.notes.as_deref().unwrap_or("").trim();

        sqlx::query(
            r#"
            INSERT INTO inventory_ledgers (
                id, item_id, movement_type, quantity, delta_quantity, balance_before, balance_after,
                unit_cost, average_unit_cost_after, total_cost_value, supplier_id, customer_id,
                installation_id, reference_no, notes, transaction_date, created_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, CURRENT_TIMESTAMP, $17)
            "#,
        )
        .bind(&ledger_id)
        .bind(&item.id)
        .bind(calc.movement_type.as_str())
        .bind(calc.quantity)
        .bind(calc.delta_quantity)
        .bind(calc.balance_before)
        .bind(calc.balance_after)
        .bind(calc.unit_cost)
        .bind(calc.average_unit_cost_after)
        .bind(calc.total_cost_value)
        .bind(&req.supplier_id)
        .bind(&req.customer_id)
        .bind(&req.installation_id)
        .bind(reference_no)
        .bind(notes)
        .bind(transaction_date)
        .bind(actor)
        .execute(&mut *tx)
        .await?;

        // 4. Update master_items balance
        sqlx::query(
            r#"
            UPDATE master_items
            SET current_stock = $2,
                average_unit_cost = $3,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $4
            WHERE id = $1
            "#,
        )
        .bind(&item.id)
        .bind(calc.balance_after)
        .bind(calc.average_unit_cost_after)
        .bind(actor)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // 5. Query created ledger row with joined names
        let created_row = sqlx::query_as::<_, InventoryLedgerRow>(
            r#"
            SELECT 
                l.id, l.item_id, i.name AS item_name, i.default_unit, l.movement_type,
                l.quantity, l.delta_quantity, l.balance_before, l.balance_after,
                l.unit_cost, l.average_unit_cost_after, l.total_cost_value, i.min_stock_level,
                l.supplier_id, s.name AS supplier_name,
                l.customer_id, c.name AS customer_name,
                l.installation_id, ins.name AS installation_name,
                l.reference_no, l.notes, l.transaction_date, l.created_at, l.created_by
            FROM inventory_ledgers l
            INNER JOIN master_items i ON i.id = l.item_id
            LEFT JOIN suppliers s ON s.id = l.supplier_id
            LEFT JOIN customers c ON c.id = l.customer_id
            LEFT JOIN installations ins ON ins.id = l.installation_id
            WHERE l.id = $1
            "#,
        )
        .bind(&ledger_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(InventoryLedgerResponse::from(created_row))
    }

    async fn record_stock_opname(
        &self,
        req: &CreateStockOpnameRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Acquire row lock
        let item = sqlx::query_as::<_, MasterItemRow>(
            r#"
            SELECT 
                i.id, i.category_id, c.name AS category_name, i.name, i.default_unit,
                i.min_stock_level, i.current_stock, i.average_unit_cost, i.is_active,
                i.created_at, i.created_by, i.modify_at, i.modify_by
            FROM master_items i
            INNER JOIN item_categories c ON c.id = i.category_id
            WHERE i.id = $1
            FOR UPDATE OF i
            "#,
        )
        .bind(&req.item_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Item with ID '{}' not found", req.item_id)))?;

        // 2. Pure calculation
        let calc = calculate_stock_opname(
            item.current_stock,
            item.average_unit_cost,
            req.actual_physical_stock,
        )?;

        // 3. Insert into inventory_ledgers
        let ledger_id = format!("inv_{}", Uuid::new_v4());
        let transaction_date = req.transaction_date.unwrap_or_else(Utc::now);
        let reference_no = req.reference_no.as_deref().unwrap_or("").trim();
        let notes = req.notes.trim();

        sqlx::query(
            r#"
            INSERT INTO inventory_ledgers (
                id, item_id, movement_type, quantity, delta_quantity, balance_before, balance_after,
                unit_cost, average_unit_cost_after, total_cost_value, supplier_id, customer_id,
                installation_id, reference_no, notes, transaction_date, created_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NULL, NULL, NULL, $11, $12, $13, CURRENT_TIMESTAMP, $14)
            "#,
        )
        .bind(&ledger_id)
        .bind(&item.id)
        .bind(calc.movement_type.as_str())
        .bind(calc.quantity)
        .bind(calc.delta_quantity)
        .bind(calc.balance_before)
        .bind(calc.balance_after)
        .bind(calc.unit_cost)
        .bind(calc.average_unit_cost_after)
        .bind(calc.total_cost_value)
        .bind(reference_no)
        .bind(notes)
        .bind(transaction_date)
        .bind(actor)
        .execute(&mut *tx)
        .await?;

        // 4. Update master_items balance
        sqlx::query(
            r#"
            UPDATE master_items
            SET current_stock = $2,
                average_unit_cost = $3,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $4
            WHERE id = $1
            "#,
        )
        .bind(&item.id)
        .bind(calc.balance_after)
        .bind(calc.average_unit_cost_after)
        .bind(actor)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // 5. Query created ledger row
        let created_row = sqlx::query_as::<_, InventoryLedgerRow>(
            r#"
            SELECT 
                l.id, l.item_id, i.name AS item_name, i.default_unit, l.movement_type,
                l.quantity, l.delta_quantity, l.balance_before, l.balance_after,
                l.unit_cost, l.average_unit_cost_after, l.total_cost_value, i.min_stock_level,
                l.supplier_id, s.name AS supplier_name,
                l.customer_id, c.name AS customer_name,
                l.installation_id, ins.name AS installation_name,
                l.reference_no, l.notes, l.transaction_date, l.created_at, l.created_by
            FROM inventory_ledgers l
            INNER JOIN master_items i ON i.id = l.item_id
            LEFT JOIN suppliers s ON s.id = l.supplier_id
            LEFT JOIN customers c ON c.id = l.customer_id
            LEFT JOIN installations ins ON ins.id = l.installation_id
            WHERE l.id = $1
            "#,
        )
        .bind(&ledger_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(InventoryLedgerResponse::from(created_row))
    }

    async fn get_stock_card(
        &self,
        item_id: &str,
        query: &StockCardQuery,
    ) -> Result<StockCardResponse, AppError> {
        let item = self
            .find_item_by_id(item_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item with ID '{item_id}' not found")))?;

        let mut count_qb =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM inventory_ledgers WHERE item_id = ");
        count_qb.push_bind(item_id);

        if let Some(m_type) = &query.movement_type {
            count_qb.push(" AND movement_type = ");
            count_qb.push_bind(m_type.as_str());
        }

        if let Some(start_date) = query.start_date {
            count_qb.push(" AND transaction_date >= ");
            count_qb.push_bind(start_date);
        }

        if let Some(end_date) = query.end_date {
            count_qb.push(" AND transaction_date <= ");
            count_qb.push_bind(end_date);
        }

        let total_movements: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;

        let mut rows_qb = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                l.id, l.item_id, i.name AS item_name, i.default_unit, l.movement_type,
                l.quantity, l.delta_quantity, l.balance_before, l.balance_after,
                l.unit_cost, l.average_unit_cost_after, l.total_cost_value, i.min_stock_level,
                l.supplier_id, s.name AS supplier_name,
                l.customer_id, c.name AS customer_name,
                l.installation_id, ins.name AS installation_name,
                l.reference_no, l.notes, l.transaction_date, l.created_at, l.created_by
            FROM inventory_ledgers l
            INNER JOIN master_items i ON i.id = l.item_id
            LEFT JOIN suppliers s ON s.id = l.supplier_id
            LEFT JOIN customers c ON c.id = l.customer_id
            LEFT JOIN installations ins ON ins.id = l.installation_id
            WHERE l.item_id = 
            "#,
        );
        rows_qb.push_bind(item_id);

        if let Some(m_type) = &query.movement_type {
            rows_qb.push(" AND l.movement_type = ");
            rows_qb.push_bind(m_type.as_str());
        }

        if let Some(start_date) = query.start_date {
            rows_qb.push(" AND l.transaction_date >= ");
            rows_qb.push_bind(start_date);
        }

        if let Some(end_date) = query.end_date {
            rows_qb.push(" AND l.transaction_date <= ");
            rows_qb.push_bind(end_date);
        }

        let is_asc = query
            .sort_order
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("asc"))
            .unwrap_or(false);

        if is_asc {
            rows_qb.push(" ORDER BY l.transaction_date ASC, l.created_at ASC, l.id ASC");
        } else {
            rows_qb.push(" ORDER BY l.transaction_date DESC, l.created_at DESC, l.id DESC");
        }

        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);

        rows_qb.push(" LIMIT ");
        rows_qb.push_bind(limit);
        rows_qb.push(" OFFSET ");
        rows_qb.push_bind(offset);

        let rows = rows_qb
            .build_query_as::<InventoryLedgerRow>()
            .fetch_all(&self.pool)
            .await?;

        let movements = rows
            .into_iter()
            .map(InventoryLedgerResponse::from)
            .collect();

        Ok(StockCardResponse {
            item,
            total_movements,
            movements,
        })
    }
}
