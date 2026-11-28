use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

use crate::common::error::AppError;
use crate::modules::finance::hpp::dto::{
    HppPurchaseCalcInput, HppSaleCalcInput, HppSessionCalcInput,
};

#[async_trait]
pub trait HppRepository: Send + Sync {
    async fn find_all_sessions(&self) -> Result<Vec<HppSessionCalcInput>, AppError>;
    async fn find_session_by_id(
        &self,
        session_id: &str,
    ) -> Result<Option<HppSessionCalcInput>, AppError>;
    async fn find_all_purchases(&self) -> Result<Vec<HppPurchaseCalcInput>, AppError>;
    async fn find_all_sales(&self) -> Result<Vec<HppSaleCalcInput>, AppError>;
    async fn lock_snapshot(
        &self,
        session_id: &str,
        final_hpp: Decimal,
        final_revenue: Decimal,
        final_shipping_subsidy: Decimal,
        final_net_profit: Decimal,
        actor: &str,
    ) -> Result<(), AppError>;
}

pub struct PgHppRepository {
    pool: PgPool,
}

impl PgHppRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HppRepository for PgHppRepository {
    async fn find_all_sessions(&self) -> Result<Vec<HppSessionCalcInput>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                ps.id, ps.session_name, ps.status, ps.start_date, ps.harvest_date,
                ps.total_holes, ps.mortality_count,
                COALESCE(ps.actual_yield_kg, 0.00) AS actual_yield_kg,
                ps.final_hpp, ps.final_revenue, ps.final_shipping_subsidy, ps.final_net_profit,
                ps.hpp_snapshot_locked_at, ps.hpp_snapshot_locked_by,
                ps.location_id, l.name AS location_name,
                pt.name AS plant_name, pt.variety AS plant_variety
            FROM planting_sessions ps
            JOIN locations l ON l.id = ps.location_id
            JOIN plant_types pt ON pt.id = ps.plant_type_id
            WHERE ps.status != 'Cancelled'
            ORDER BY ps.start_date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|r| HppSessionCalcInput {
                session_id: r.get("id"),
                session_name: r.get("session_name"),
                plant_name: r.get("plant_name"),
                plant_variety: r.get("plant_variety"),
                location_id: r.get("location_id"),
                location_name: r.get("location_name"),
                status: r.get("status"),
                start_date: r.get("start_date"),
                harvest_date: r.get("harvest_date"),
                total_holes: r.get("total_holes"),
                mortality_count: r.get("mortality_count"),
                actual_yield_kg: r
                    .get::<Option<Decimal>, _>("actual_yield_kg")
                    .unwrap_or(Decimal::ZERO),
                final_hpp: r.get("final_hpp"),
                final_revenue: r.get("final_revenue"),
                final_shipping_subsidy: r.get("final_shipping_subsidy"),
                final_net_profit: r.get("final_net_profit"),
                hpp_snapshot_locked_at: r.get("hpp_snapshot_locked_at"),
                hpp_snapshot_locked_by: r.get("hpp_snapshot_locked_by"),
            })
            .collect();

        Ok(items)
    }

    async fn find_session_by_id(
        &self,
        session_id: &str,
    ) -> Result<Option<HppSessionCalcInput>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                ps.id, ps.session_name, ps.status, ps.start_date, ps.harvest_date,
                ps.total_holes, ps.mortality_count,
                COALESCE(ps.actual_yield_kg, 0.00) AS actual_yield_kg,
                ps.final_hpp, ps.final_revenue, ps.final_shipping_subsidy, ps.final_net_profit,
                ps.hpp_snapshot_locked_at, ps.hpp_snapshot_locked_by,
                ps.location_id, l.name AS location_name,
                pt.name AS plant_name, pt.variety AS plant_variety
            FROM planting_sessions ps
            JOIN locations l ON l.id = ps.location_id
            JOIN plant_types pt ON pt.id = ps.plant_type_id
            WHERE ps.id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| HppSessionCalcInput {
            session_id: r.get("id"),
            session_name: r.get("session_name"),
            plant_name: r.get("plant_name"),
            plant_variety: r.get("plant_variety"),
            location_id: r.get("location_id"),
            location_name: r.get("location_name"),
            status: r.get("status"),
            start_date: r.get("start_date"),
            harvest_date: r.get("harvest_date"),
            total_holes: r.get("total_holes"),
            mortality_count: r.get("mortality_count"),
            actual_yield_kg: r
                .get::<Option<Decimal>, _>("actual_yield_kg")
                .unwrap_or(Decimal::ZERO),
            final_hpp: r.get("final_hpp"),
            final_revenue: r.get("final_revenue"),
            final_shipping_subsidy: r.get("final_shipping_subsidy"),
            final_net_profit: r.get("final_net_profit"),
            hpp_snapshot_locked_at: r.get("hpp_snapshot_locked_at"),
            hpp_snapshot_locked_by: r.get("hpp_snapshot_locked_by"),
        }))
    }

    async fn find_all_purchases(&self) -> Result<Vec<HppPurchaseCalcInput>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                pi.id AS purchase_item_id,
                pi.purchase_id,
                ph.invoice_number,
                ph.transaction_date,
                ph.location_id,
                pi.item_name,
                pi.category,
                pi.unit,
                pi.quantity,
                pi.total_cost,
                pi.allocation_type,
                pi.allocation_days,
                pi.session_id
            FROM purchase_items pi
            JOIN purchase_headers ph ON ph.id = pi.purchase_id
            ORDER BY ph.transaction_date ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|r| HppPurchaseCalcInput {
                purchase_item_id: r.get("purchase_item_id"),
                purchase_id: r.get("purchase_id"),
                invoice_number: r.get("invoice_number"),
                transaction_date: r.get("transaction_date"),
                location_id: r.get("location_id"),
                item_name: r.get("item_name"),
                category: r.get("category"),
                unit: r.get("unit"),
                quantity: r.get("quantity"),
                total_cost: r.get("total_cost"),
                allocation_type: r.get("allocation_type"),
                allocation_days: r.get("allocation_days"),
                session_id: r.get("session_id"),
            })
            .collect();

        Ok(items)
    }

    async fn find_all_sales(&self) -> Result<Vec<HppSaleCalcInput>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                s.id AS sale_id,
                s.sale_number,
                s.sale_date,
                s.session_id,
                COALESCE(NULLIF(TRIM(c.name), ''), NULLIF(TRIM(s.customer_name_snapshot), ''), 'Pelanggan Umum') AS customer_name,
                s.unit,
                s.quantity,
                s.price_per_unit,
                s.gross_revenue,
                s.discount,
                s.total_revenue,
                s.seller_shipping_subsidy,
                s.net_proceeds,
                s.payment_status
            FROM sales s
            LEFT JOIN customers c ON c.id = s.customer_id
            WHERE s.status = 'Realized'
            ORDER BY s.sale_date ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|r| HppSaleCalcInput {
                sale_id: r.get("sale_id"),
                sale_number: r.get("sale_number"),
                sale_date: r.get("sale_date"),
                session_id: r.get("session_id"),
                customer_name: r.get("customer_name"),
                unit: r.get("unit"),
                quantity: r.get("quantity"),
                price_per_unit: r.get("price_per_unit"),
                gross_revenue: r.get("gross_revenue"),
                discount: r.get("discount"),
                total_revenue: r.get("total_revenue"),
                seller_shipping_subsidy: r.get("seller_shipping_subsidy"),
                net_proceeds: r.get("net_proceeds"),
                payment_status: r.get("payment_status"),
            })
            .collect();

        Ok(items)
    }

    async fn lock_snapshot(
        &self,
        session_id: &str,
        final_hpp: Decimal,
        final_revenue: Decimal,
        final_shipping_subsidy: Decimal,
        final_net_profit: Decimal,
        actor: &str,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query(
            "SELECT status, hpp_snapshot_locked_at FROM planting_sessions WHERE id = $1 FOR UPDATE",
        )
        .bind(session_id)
        .fetch_optional(&mut *tx)
        .await?;

        let s = row.ok_or_else(|| {
            AppError::NotFound(format!(
                "Sesi tanam dengan ID '{session_id}' tidak ditemukan"
            ))
        })?;

        let status: String = s.get("status");
        let locked_at: Option<chrono::DateTime<chrono::Utc>> = s.get("hpp_snapshot_locked_at");

        if !status.eq_ignore_ascii_case("Completed") {
            return Err(AppError::BadRequest(
                "Hanya sesi tanam berstatus 'Completed' yang dapat dikunci snapshot HPP-nya".into(),
            ));
        }

        if locked_at.is_some() {
            return Err(AppError::BadRequest(
                "Snapshot HPP untuk sesi ini sudah pernah dikunci secara permanen".into(),
            ));
        }

        sqlx::query(
            r#"
            UPDATE planting_sessions
            SET
                final_hpp = $1,
                final_revenue = $2,
                final_shipping_subsidy = $3,
                final_net_profit = $4,
                hpp_snapshot_locked_at = CURRENT_TIMESTAMP,
                hpp_snapshot_locked_by = $5,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $5
            WHERE id = $6
            "#,
        )
        .bind(final_hpp)
        .bind(final_revenue)
        .bind(final_shipping_subsidy)
        .bind(final_net_profit)
        .bind(actor)
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
