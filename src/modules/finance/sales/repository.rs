use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::modules::finance::purchases::service::format_document_number;
use crate::modules::finance::sales::dto::{
    CreateSaleRequest, PaymentStatus, SaleFinancialResult, SaleKpi, SaleListResponse, SaleQuery,
    SaleResponse, SaleStatus, ShippingBorneBy, UpdateSaleRequest,
};

#[async_trait]
pub trait SaleRepository: Send + Sync {
    async fn find_all(&self, query: &SaleQuery) -> Result<SaleListResponse, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<SaleResponse>, AppError>;
    async fn create(
        &self,
        req: CreateSaleRequest,
        fin: SaleFinancialResult,
        sale_date: NaiveDate,
        actor: &str,
    ) -> Result<SaleResponse, AppError>;
    async fn update(
        &self,
        id: &str,
        req: UpdateSaleRequest,
        fin: SaleFinancialResult,
        sale_date: NaiveDate,
        actor: &str,
    ) -> Result<SaleResponse, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PgSaleRepository {
    pool: PgPool,
}

impl PgSaleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_sale_response(&self, r: &sqlx::postgres::PgRow) -> SaleResponse {
        let status_str: String = r.get("status");
        let status = SaleStatus::from_str(&status_str).unwrap_or(SaleStatus::Order);

        let payment_str: String = r.get("payment_status");
        let payment_status =
            PaymentStatus::from_str(&payment_str).unwrap_or(PaymentStatus::Piutang);

        let shipping_borne_str: String = r.get("shipping_borne_by");
        let shipping_borne_by =
            ShippingBorneBy::from_str(&shipping_borne_str).unwrap_or(ShippingBorneBy::Pembeli);

        SaleResponse {
            id: r.get("id"),
            sale_number: r.get("sale_number"),
            sale_date: r.get("sale_date"),
            session_id: r.get("session_id"),
            session_name: r.get("session_name"),
            plant_name: r.get("plant_name"),
            location_id: r.get("location_id"),
            location_name: r.get("location_name"),
            customer_id: r.get("customer_id"),
            customer_name: r.get("customer_name"),
            sales_channel: r.get("sales_channel"),
            unit: r.get("unit"),
            requested_quantity: r.get("requested_quantity"),
            quantity: r.get("quantity"),
            fulfillment_rate_percent: r.get("fulfillment_rate_percent"),
            price_per_unit: r.get("price_per_unit"),
            gross_revenue: r.get("gross_revenue"),
            discount: r.get("discount"),
            total_revenue: r.get("total_revenue"),
            shipping_cost: r.get("shipping_cost"),
            shipping_borne_by,
            seller_shipping_subsidy: r.get("seller_shipping_subsidy"),
            net_proceeds: r.get("net_proceeds"),
            status,
            payment_status,
            paid_at: r.get("paid_at"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            created_by: r.get("created_by"),
            modify_at: r.get("modify_at"),
            modify_by: r.get("modify_by"),
        }
    }
}

#[async_trait]
impl SaleRepository for PgSaleRepository {
    async fn find_all(&self, query: &SaleQuery) -> Result<SaleListResponse, AppError> {
        let mut qb = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                s.id,
                s.sale_number,
                s.sale_date,
                s.session_id,
                ps.session_name,
                p.name AS plant_name,
                s.location_id,
                l.name AS location_name,
                s.customer_id,
                COALESCE(c.name, s.customer_name_snapshot) AS customer_name,
                s.sales_channel,
                s.unit,
                s.requested_quantity,
                s.quantity,
                s.fulfillment_rate_percent,
                s.price_per_unit,
                s.gross_revenue,
                s.discount,
                s.total_revenue,
                s.shipping_cost,
                s.shipping_borne_by,
                s.seller_shipping_subsidy,
                s.net_proceeds,
                s.status,
                s.payment_status,
                s.paid_at,
                s.notes,
                s.created_at,
                s.created_by,
                s.modify_at,
                s.modify_by
            FROM sales s
            JOIN planting_sessions ps ON ps.id = s.session_id
            JOIN plant_types p ON p.id = ps.plant_type_id
            JOIN locations l ON l.id = s.location_id
            LEFT JOIN customers c ON c.id = s.customer_id
            WHERE 1=1
            "#,
        );

        if let Some(sess_id) = &query.session_id {
            qb.push(" AND s.session_id = ").push_bind(sess_id);
        }
        if let Some(loc_id) = &query.location_id {
            qb.push(" AND s.location_id = ").push_bind(loc_id);
        }
        if let Some(cus_id) = &query.customer_id {
            qb.push(" AND s.customer_id = ").push_bind(cus_id);
        }
        if let Some(st) = query.status {
            qb.push(" AND s.status = ").push_bind(st.to_string());
        }
        if let Some(ps) = query.payment_status {
            qb.push(" AND s.payment_status = ")
                .push_bind(ps.to_string());
        }
        if let Some(s_date) = query.start_date {
            qb.push(" AND s.sale_date >= ").push_bind(s_date);
        }
        if let Some(e_date) = query.end_date {
            qb.push(" AND s.sale_date <= ").push_bind(e_date);
        }
        if let Some(search) = &query.search {
            let trimmed = search.trim();
            if !trimmed.is_empty() {
                let pattern = format!("%{trimmed}%");
                qb.push(" AND (")
                    .push("s.sale_number ILIKE ")
                    .push_bind(pattern.clone())
                    .push(" OR s.customer_name_snapshot ILIKE ")
                    .push_bind(pattern.clone())
                    .push(" OR COALESCE(c.name, '') ILIKE ")
                    .push_bind(pattern.clone())
                    .push(" OR ps.session_name ILIKE ")
                    .push_bind(pattern.clone())
                    .push(" OR s.sales_channel ILIKE ")
                    .push_bind(pattern)
                    .push(")");
            }
        }

        let is_asc = query.sort_order.as_deref() == Some("asc");
        if is_asc {
            qb.push(" ORDER BY s.sale_date ASC, s.created_at ASC");
        } else {
            qb.push(" ORDER BY s.sale_date DESC, s.created_at DESC");
        }

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut total_realized_revenue = Decimal::ZERO;
        let mut total_receivable_amount = Decimal::ZERO;
        let mut total_sold_kg = Decimal::ZERO;
        let mut total_shipping_subsidy = Decimal::ZERO;
        let mut order_pipeline_count = 0i64;

        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let item = self.row_to_sale_response(&row);

            if item.status == SaleStatus::Realized {
                total_realized_revenue += item.total_revenue;
                total_sold_kg += item.quantity;
                total_shipping_subsidy += item.seller_shipping_subsidy;
            } else if item.status == SaleStatus::Order {
                order_pipeline_count += 1;
            }

            if item.payment_status == PaymentStatus::Piutang {
                total_receivable_amount += item.total_revenue;
            }

            items.push(item);
        }

        Ok(SaleListResponse {
            kpi: SaleKpi {
                total_realized_revenue,
                total_receivable_amount,
                total_sold_kg,
                total_shipping_subsidy,
                order_pipeline_count,
            },
            items,
        })
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<SaleResponse>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                s.id,
                s.sale_number,
                s.sale_date,
                s.session_id,
                ps.session_name,
                p.name AS plant_name,
                s.location_id,
                l.name AS location_name,
                s.customer_id,
                COALESCE(c.name, s.customer_name_snapshot) AS customer_name,
                s.sales_channel,
                s.unit,
                s.requested_quantity,
                s.quantity,
                s.fulfillment_rate_percent,
                s.price_per_unit,
                s.gross_revenue,
                s.discount,
                s.total_revenue,
                s.shipping_cost,
                s.shipping_borne_by,
                s.seller_shipping_subsidy,
                s.net_proceeds,
                s.status,
                s.payment_status,
                s.paid_at,
                s.notes,
                s.created_at,
                s.created_by,
                s.modify_at,
                s.modify_by
            FROM sales s
            JOIN planting_sessions ps ON ps.id = s.session_id
            JOIN plant_types p ON p.id = ps.plant_type_id
            JOIN locations l ON l.id = s.location_id
            LEFT JOIN customers c ON c.id = s.customer_id
            WHERE s.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_sale_response(&r)))
    }

    async fn create(
        &self,
        req: CreateSaleRequest,
        fin: SaleFinancialResult,
        sale_date: NaiveDate,
        actor: &str,
    ) -> Result<SaleResponse, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Verify session: must exist and !is_locked && status != 'Completed'
        let sess_row = sqlx::query(
            "SELECT id, location_id, is_locked, status FROM planting_sessions WHERE id = $1 FOR UPDATE",
        )
        .bind(&req.session_id)
        .fetch_optional(&mut *tx)
        .await?;

        let sess = sess_row.ok_or_else(|| {
            AppError::BadRequest(format!(
                "Sesi tanam dengan ID '{}' tidak ditemukan",
                req.session_id
            ))
        })?;

        let is_locked: bool = sess.get("is_locked");
        let sess_status: String = sess.get("status");
        let sess_location_id: String = sess.get("location_id");

        if is_locked
            || sess_status.eq_ignore_ascii_case("Completed")
            || sess_status.eq_ignore_ascii_case("Cancelled")
        {
            return Err(AppError::BadRequest(
                "Tidak dapat mencatat penjualan pada sesi tanam yang sudah selesai, dibatalkan, atau terkunci"
                    .into(),
            ));
        }

        // 2. Determine and verify location_id
        let final_location_id = req.location_id.unwrap_or(sess_location_id);
        let loc_exists = sqlx::query("SELECT 1 FROM locations WHERE id = $1")
            .bind(&final_location_id)
            .fetch_optional(&mut *tx)
            .await?;
        if loc_exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "Lokasi dengan ID '{final_location_id}' tidak ditemukan"
            )));
        }

        // 3. Verify customer (if provided)
        let customer_name = if let Some(cus_id) = &req.customer_id {
            let cus_row = sqlx::query("SELECT id, name FROM customers WHERE id = $1")
                .bind(cus_id)
                .fetch_optional(&mut *tx)
                .await?;
            let c = cus_row.ok_or_else(|| {
                AppError::BadRequest(format!("Pelanggan dengan ID '{cus_id}' tidak ditemukan"))
            })?;
            c.get::<String, _>("name")
        } else {
            req.customer_name.unwrap_or_default()
        };

        // 4. Generate sale_number with transaction-scoped advisory lock
        let prefix = format!("SLS-{}", sale_date.format("%Y%m%d"));
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(&prefix)
            .execute(&mut *tx)
            .await?;

        let like_pattern = format!("{prefix}-%");
        let max_seq_row = sqlx::query(
            r#"
            SELECT COALESCE(MAX(CAST(SPLIT_PART(sale_number, '-', 3) AS INTEGER)), 0) + 1 AS next_seq
            FROM sales
            WHERE sale_number LIKE $1
            "#,
        )
        .bind(&like_pattern)
        .fetch_one(&mut *tx)
        .await?;

        let next_seq: i32 = max_seq_row.get("next_seq");
        let sale_number = format_document_number("SLS", sale_date, next_seq);

        let sale_id = format!("sls_{}", Uuid::new_v4());
        let status = req.status.unwrap_or(SaleStatus::Order);
        let payment_status = req.payment_status.unwrap_or(PaymentStatus::Piutang);
        let paid_at: Option<DateTime<Utc>> = if payment_status == PaymentStatus::Lunas {
            Some(Utc::now())
        } else {
            None
        };

        let unit = req.unit.as_deref().unwrap_or("KG").trim().to_uppercase();
        let sales_channel = req
            .sales_channel
            .as_deref()
            .unwrap_or("Direct")
            .trim()
            .to_string();
        let shipping_borne = req.shipping_borne_by.unwrap_or(ShippingBorneBy::Pembeli);

        sqlx::query(
            r#"
            INSERT INTO sales (
                id, sale_number, sale_date, session_id, location_id,
                customer_id, customer_name_snapshot, sales_channel, unit,
                requested_quantity, quantity, fulfillment_rate_percent,
                price_per_unit, gross_revenue, discount, total_revenue,
                shipping_cost, shipping_borne_by, seller_shipping_subsidy,
                net_proceeds, status, payment_status, paid_at,
                notes, created_by, modify_by
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21, $22, $23,
                $24, $25, $25
            )
            "#,
        )
        .bind(&sale_id)
        .bind(&sale_number)
        .bind(sale_date)
        .bind(&req.session_id)
        .bind(&final_location_id)
        .bind(&req.customer_id)
        .bind(&customer_name)
        .bind(&sales_channel)
        .bind(&unit)
        .bind(fin.requested_quantity)
        .bind(fin.quantity)
        .bind(fin.fulfillment_rate_percent)
        .bind(req.price_per_unit)
        .bind(fin.gross_revenue)
        .bind(fin.discount)
        .bind(fin.total_revenue)
        .bind(fin.shipping_cost)
        .bind(shipping_borne.to_string())
        .bind(fin.seller_shipping_subsidy)
        .bind(fin.net_proceeds)
        .bind(status.to_string())
        .bind(payment_status.to_string())
        .bind(paid_at)
        .bind(req.notes.as_deref().unwrap_or(""))
        .bind(actor)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let res = self.find_by_id(&sale_id).await?.ok_or_else(|| {
            AppError::Internal("Gagal memuat transaksi penjualan yang baru dibuat".into())
        })?;

        Ok(res)
    }

    async fn update(
        &self,
        id: &str,
        req: UpdateSaleRequest,
        fin: SaleFinancialResult,
        sale_date: NaiveDate,
        actor: &str,
    ) -> Result<SaleResponse, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Lock existing sale
        let existing_row = sqlx::query(
            "SELECT session_id, location_id, status, payment_status, paid_at FROM sales WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let ex = existing_row
            .ok_or_else(|| AppError::NotFound(format!("Sale with ID '{id}' not found")))?;
        let ex_session_id: String = ex.get("session_id");
        let ex_location_id: String = ex.get("location_id");
        let ex_paid_at: Option<DateTime<Utc>> = ex.get("paid_at");

        // 2. Check existing session immutability
        let ex_sess_row =
            sqlx::query("SELECT is_locked, status FROM planting_sessions WHERE id = $1")
                .bind(&ex_session_id)
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(s) = ex_sess_row {
            let is_locked: bool = s.get("is_locked");
            let sess_status: String = s.get("status");
            if is_locked
                || sess_status.eq_ignore_ascii_case("Completed")
                || sess_status.eq_ignore_ascii_case("Cancelled")
            {
                return Err(AppError::BadRequest(
                    "Tidak dapat mengubah transaksi penjualan pada sesi tanam yang sudah selesai, dibatalkan, atau terkunci".into(),
                ));
            }
        }

        // 3. If target session_id changes, verify new session
        let target_location_id = if req.session_id != ex_session_id {
            let new_sess_row = sqlx::query(
                "SELECT location_id, is_locked, status FROM planting_sessions WHERE id = $1 FOR UPDATE",
            )
            .bind(&req.session_id)
            .fetch_optional(&mut *tx)
            .await?;

            let ns = new_sess_row.ok_or_else(|| {
                AppError::BadRequest(format!(
                    "Sesi tanam tujuan '{}' tidak ditemukan",
                    req.session_id
                ))
            })?;

            let is_locked: bool = ns.get("is_locked");
            let sess_status: String = ns.get("status");
            if is_locked
                || sess_status.eq_ignore_ascii_case("Completed")
                || sess_status.eq_ignore_ascii_case("Cancelled")
            {
                return Err(AppError::BadRequest(
                    "Sesi tanam tujuan sudah selesai, dibatalkan, atau terkunci".into(),
                ));
            }
            req.location_id.unwrap_or_else(|| ns.get("location_id"))
        } else {
            req.location_id.unwrap_or(ex_location_id)
        };

        // 4. Verify location exists
        let loc_exists = sqlx::query("SELECT 1 FROM locations WHERE id = $1")
            .bind(&target_location_id)
            .fetch_optional(&mut *tx)
            .await?;
        if loc_exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "Lokasi dengan ID '{target_location_id}' tidak ditemukan"
            )));
        }

        // 5. Verify customer
        let customer_name = if let Some(cus_id) = &req.customer_id {
            let cus_row = sqlx::query("SELECT id, name FROM customers WHERE id = $1")
                .bind(cus_id)
                .fetch_optional(&mut *tx)
                .await?;
            let c = cus_row.ok_or_else(|| {
                AppError::BadRequest(format!("Pelanggan dengan ID '{cus_id}' tidak ditemukan"))
            })?;
            c.get::<String, _>("name")
        } else {
            req.customer_name.unwrap_or_default()
        };

        let status = req.status.unwrap_or(SaleStatus::Order);
        let payment_status = req.payment_status.unwrap_or(PaymentStatus::Piutang);
        let paid_at = match payment_status {
            PaymentStatus::Lunas => {
                if ex_paid_at.is_some() {
                    ex_paid_at
                } else {
                    Some(Utc::now())
                }
            }
            PaymentStatus::Piutang => None,
        };

        let unit = req.unit.as_deref().unwrap_or("KG").trim().to_uppercase();
        let sales_channel = req
            .sales_channel
            .as_deref()
            .unwrap_or("Direct")
            .trim()
            .to_string();
        let shipping_borne = req.shipping_borne_by.unwrap_or(ShippingBorneBy::Pembeli);

        sqlx::query(
            r#"
            UPDATE sales SET
                sale_date = $1,
                session_id = $2,
                location_id = $3,
                customer_id = $4,
                customer_name_snapshot = $5,
                sales_channel = $6,
                unit = $7,
                requested_quantity = $8,
                quantity = $9,
                fulfillment_rate_percent = $10,
                price_per_unit = $11,
                gross_revenue = $12,
                discount = $13,
                total_revenue = $14,
                shipping_cost = $15,
                shipping_borne_by = $16,
                seller_shipping_subsidy = $17,
                net_proceeds = $18,
                status = $19,
                payment_status = $20,
                paid_at = $21,
                notes = $22,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = $23
            WHERE id = $24
            "#,
        )
        .bind(sale_date)
        .bind(&req.session_id)
        .bind(&target_location_id)
        .bind(&req.customer_id)
        .bind(&customer_name)
        .bind(&sales_channel)
        .bind(&unit)
        .bind(fin.requested_quantity)
        .bind(fin.quantity)
        .bind(fin.fulfillment_rate_percent)
        .bind(req.price_per_unit)
        .bind(fin.gross_revenue)
        .bind(fin.discount)
        .bind(fin.total_revenue)
        .bind(fin.shipping_cost)
        .bind(shipping_borne.to_string())
        .bind(fin.seller_shipping_subsidy)
        .bind(fin.net_proceeds)
        .bind(status.to_string())
        .bind(payment_status.to_string())
        .bind(paid_at)
        .bind(req.notes.as_deref().unwrap_or(""))
        .bind(actor)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let res = self.find_by_id(id).await?.ok_or_else(|| {
            AppError::Internal("Gagal memuat transaksi penjualan setelah diperbarui".into())
        })?;

        Ok(res)
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        let sale_row = sqlx::query("SELECT session_id FROM sales WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;

        let s =
            sale_row.ok_or_else(|| AppError::NotFound(format!("Sale with ID '{id}' not found")))?;
        let session_id: String = s.get("session_id");

        let sess_row = sqlx::query("SELECT is_locked, status FROM planting_sessions WHERE id = $1")
            .bind(&session_id)
            .fetch_optional(&mut *tx)
            .await?;

        if let Some(sess) = sess_row {
            let is_locked: bool = sess.get("is_locked");
            let status: String = sess.get("status");
            if is_locked
                || status.eq_ignore_ascii_case("Completed")
                || status.eq_ignore_ascii_case("Cancelled")
            {
                return Err(AppError::BadRequest(
                    "Tidak dapat menghapus transaksi penjualan pada sesi tanam yang sudah selesai, dibatalkan, atau terkunci".into(),
                ));
            }
        }

        sqlx::query("DELETE FROM sales WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
