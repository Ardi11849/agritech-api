use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::modules::finance::purchases::dto::{
    AllocationType, CreatePurchaseRequest, PurchaseDistributionResult, PurchaseItemResponse,
    PurchaseKpi, PurchaseListResponse, PurchaseQuery, PurchaseResponse,
};
use crate::modules::finance::purchases::service::format_document_number;
use crate::modules::inventory::dto::MovementType;
use crate::modules::inventory::service::calculate_stock_movement;

#[async_trait]
pub trait PurchaseRepository: Send + Sync {
    async fn find_all(&self, query: &PurchaseQuery) -> Result<PurchaseListResponse, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<PurchaseResponse>, AppError>;
    async fn create(
        &self,
        req: CreatePurchaseRequest,
        dist: PurchaseDistributionResult,
        tx_date: NaiveDate,
        actor: &str,
    ) -> Result<PurchaseResponse, AppError>;
    async fn delete(&self, id: &str, actor: &str) -> Result<(), AppError>;
}

pub struct PgPurchaseRepository {
    pool: PgPool,
}

impl PgPurchaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseRepository for PgPurchaseRepository {
    async fn find_all(&self, query: &PurchaseQuery) -> Result<PurchaseListResponse, AppError> {
        let mut qb = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                h.id,
                h.invoice_number,
                h.transaction_date,
                h.supplier_id,
                COALESCE(s.name, h.supplier_name_snapshot) AS supplier_name,
                h.location_id,
                l.name AS location_name,
                h.subtotal_amount,
                h.global_discount,
                h.global_shipping,
                h.total_amount,
                h.notes,
                h.created_at,
                h.created_by,
                h.modify_at,
                h.modify_by
            FROM purchase_headers h
            LEFT JOIN suppliers s ON s.id = h.supplier_id
            JOIN locations l ON l.id = h.location_id
            WHERE 1=1
            "#,
        );

        if let Some(loc_id) = &query.location_id {
            qb.push(" AND h.location_id = ").push_bind(loc_id);
        }
        if let Some(sup_id) = &query.supplier_id {
            qb.push(" AND h.supplier_id = ").push_bind(sup_id);
        }
        if let Some(s_date) = query.start_date {
            qb.push(" AND h.transaction_date >= ").push_bind(s_date);
        }
        if let Some(e_date) = query.end_date {
            qb.push(" AND h.transaction_date <= ").push_bind(e_date);
        }
        if let Some(alloc_type) = query.allocation_type {
            qb.push(" AND EXISTS (SELECT 1 FROM purchase_items pi WHERE pi.purchase_id = h.id AND pi.allocation_type = ")
                .push_bind(alloc_type.to_string())
                .push(")");
        }
        if let Some(sess_id) = &query.session_id {
            qb.push(" AND EXISTS (SELECT 1 FROM purchase_items pi WHERE pi.purchase_id = h.id AND pi.session_id = ")
                .push_bind(sess_id)
                .push(")");
        }
        if let Some(search) = &query.search {
            let trimmed = search.trim();
            if !trimmed.is_empty() {
                let pattern = format!("%{trimmed}%");
                qb.push(" AND (")
                    .push("h.invoice_number ILIKE ").push_bind(pattern.clone())
                    .push(" OR h.supplier_name_snapshot ILIKE ").push_bind(pattern.clone())
                    .push(" OR COALESCE(s.name, '') ILIKE ").push_bind(pattern.clone())
                    .push(" OR l.name ILIKE ").push_bind(pattern.clone())
                    .push(" OR EXISTS (SELECT 1 FROM purchase_items pi WHERE pi.purchase_id = h.id AND pi.item_name ILIKE ")
                    .push_bind(pattern)
                    .push("))");
            }
        }

        let is_asc = query.sort_order.as_deref() == Some("asc");
        if is_asc {
            qb.push(" ORDER BY h.transaction_date ASC, h.created_at ASC");
        } else {
            qb.push(" ORDER BY h.transaction_date DESC, h.created_at DESC");
        }

        let header_rows = qb.build().fetch_all(&self.pool).await?;
        if header_rows.is_empty() {
            return Ok(PurchaseListResponse {
                kpi: PurchaseKpi {
                    total_purchases_count: 0,
                    total_spend_amount: Decimal::ZERO,
                    direct_cost_total: Decimal::ZERO,
                    shared_and_asset_cost_total: Decimal::ZERO,
                },
                items: Vec::new(),
            });
        }

        let header_ids: Vec<String> = header_rows.iter().map(|r| r.get("id")).collect();

        let item_rows = sqlx::query(
            r#"
            SELECT
                pi.id,
                pi.purchase_id,
                pi.line_number,
                pi.item_id,
                pi.item_name,
                pi.category,
                pi.unit,
                pi.quantity,
                pi.price_per_unit,
                pi.subtotal,
                pi.discount,
                pi.shipping_cost,
                pi.total_cost,
                pi.effective_unit_cost,
                pi.allocation_type,
                pi.session_id,
                ps.session_name,
                pi.allocation_days,
                pi.daily_allocation_rate,
                pi.is_synced_to_inventory
            FROM purchase_items pi
            LEFT JOIN planting_sessions ps ON ps.id = pi.session_id
            WHERE pi.purchase_id = ANY($1)
            ORDER BY pi.purchase_id, pi.line_number ASC
            "#,
        )
        .bind(&header_ids)
        .fetch_all(&self.pool)
        .await?;

        let mut items_map: HashMap<String, Vec<PurchaseItemResponse>> = HashMap::new();
        let mut direct_cost_total = Decimal::ZERO;
        let mut shared_and_asset_cost_total = Decimal::ZERO;

        for r in item_rows {
            let pid: String = r.get("purchase_id");
            let alloc_str: String = r.get("allocation_type");
            let alloc_type = AllocationType::from_str(&alloc_str).unwrap_or(AllocationType::Shared);
            let total_cost: Decimal = r.get("total_cost");

            match alloc_type {
                AllocationType::Direct => direct_cost_total += total_cost,
                AllocationType::Shared | AllocationType::Asset => {
                    shared_and_asset_cost_total += total_cost
                }
            }

            let item_resp = PurchaseItemResponse {
                id: r.get("id"),
                purchase_id: pid.clone(),
                line_number: r.get("line_number"),
                item_id: r.get("item_id"),
                item_name: r.get("item_name"),
                category: r.get("category"),
                unit: r.get("unit"),
                quantity: r.get("quantity"),
                price_per_unit: r.get("price_per_unit"),
                subtotal: r.get("subtotal"),
                discount: r.get("discount"),
                shipping_cost: r.get("shipping_cost"),
                total_cost,
                effective_unit_cost: r.get("effective_unit_cost"),
                allocation_type: alloc_type,
                session_id: r.get("session_id"),
                session_name: r.get("session_name"),
                allocation_days: r.get("allocation_days"),
                daily_allocation_rate: r.get("daily_allocation_rate"),
                is_synced_to_inventory: r.get("is_synced_to_inventory"),
            };

            items_map.entry(pid).or_default().push(item_resp);
        }

        let mut total_spend_amount = Decimal::ZERO;
        let mut purchases = Vec::with_capacity(header_rows.len());

        for r in header_rows {
            let hid: String = r.get("id");
            let tot: Decimal = r.get("total_amount");
            total_spend_amount += tot;

            let items = items_map.remove(&hid).unwrap_or_default();
            let item_count = items.len() as i32;

            purchases.push(PurchaseResponse {
                id: hid,
                invoice_number: r.get("invoice_number"),
                transaction_date: r.get("transaction_date"),
                supplier_id: r.get("supplier_id"),
                supplier_name: r.get("supplier_name"),
                location_id: r.get("location_id"),
                location_name: r.get("location_name"),
                subtotal_amount: r.get("subtotal_amount"),
                global_discount: r.get("global_discount"),
                global_shipping: r.get("global_shipping"),
                total_amount: tot,
                item_count,
                notes: r.get("notes"),
                items,
                created_at: r.get("created_at"),
                created_by: r.get("created_by"),
                modify_at: r.get("modify_at"),
                modify_by: r.get("modify_by"),
            });
        }

        Ok(PurchaseListResponse {
            kpi: PurchaseKpi {
                total_purchases_count: purchases.len() as i64,
                total_spend_amount,
                direct_cost_total,
                shared_and_asset_cost_total,
            },
            items: purchases,
        })
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<PurchaseResponse>, AppError> {
        let header_row = sqlx::query(
            r#"
            SELECT
                h.id,
                h.invoice_number,
                h.transaction_date,
                h.supplier_id,
                COALESCE(s.name, h.supplier_name_snapshot) AS supplier_name,
                h.location_id,
                l.name AS location_name,
                h.subtotal_amount,
                h.global_discount,
                h.global_shipping,
                h.total_amount,
                h.notes,
                h.created_at,
                h.created_by,
                h.modify_at,
                h.modify_by
            FROM purchase_headers h
            LEFT JOIN suppliers s ON s.id = h.supplier_id
            JOIN locations l ON l.id = h.location_id
            WHERE h.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let r = match header_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let item_rows = sqlx::query(
            r#"
            SELECT
                pi.id,
                pi.purchase_id,
                pi.line_number,
                pi.item_id,
                pi.item_name,
                pi.category,
                pi.unit,
                pi.quantity,
                pi.price_per_unit,
                pi.subtotal,
                pi.discount,
                pi.shipping_cost,
                pi.total_cost,
                pi.effective_unit_cost,
                pi.allocation_type,
                pi.session_id,
                ps.session_name,
                pi.allocation_days,
                pi.daily_allocation_rate,
                pi.is_synced_to_inventory
            FROM purchase_items pi
            LEFT JOIN planting_sessions ps ON ps.id = pi.session_id
            WHERE pi.purchase_id = $1
            ORDER BY pi.line_number ASC
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<PurchaseItemResponse> = item_rows
            .into_iter()
            .map(|ir| {
                let alloc_str: String = ir.get("allocation_type");
                let alloc_type =
                    AllocationType::from_str(&alloc_str).unwrap_or(AllocationType::Shared);
                PurchaseItemResponse {
                    id: ir.get("id"),
                    purchase_id: ir.get("purchase_id"),
                    line_number: ir.get("line_number"),
                    item_id: ir.get("item_id"),
                    item_name: ir.get("item_name"),
                    category: ir.get("category"),
                    unit: ir.get("unit"),
                    quantity: ir.get("quantity"),
                    price_per_unit: ir.get("price_per_unit"),
                    subtotal: ir.get("subtotal"),
                    discount: ir.get("discount"),
                    shipping_cost: ir.get("shipping_cost"),
                    total_cost: ir.get("total_cost"),
                    effective_unit_cost: ir.get("effective_unit_cost"),
                    allocation_type: alloc_type,
                    session_id: ir.get("session_id"),
                    session_name: ir.get("session_name"),
                    allocation_days: ir.get("allocation_days"),
                    daily_allocation_rate: ir.get("daily_allocation_rate"),
                    is_synced_to_inventory: ir.get("is_synced_to_inventory"),
                }
            })
            .collect();

        let item_count = items.len() as i32;

        Ok(Some(PurchaseResponse {
            id: r.get("id"),
            invoice_number: r.get("invoice_number"),
            transaction_date: r.get("transaction_date"),
            supplier_id: r.get("supplier_id"),
            supplier_name: r.get("supplier_name"),
            location_id: r.get("location_id"),
            location_name: r.get("location_name"),
            subtotal_amount: r.get("subtotal_amount"),
            global_discount: r.get("global_discount"),
            global_shipping: r.get("global_shipping"),
            total_amount: r.get("total_amount"),
            item_count,
            notes: r.get("notes"),
            items,
            created_at: r.get("created_at"),
            created_by: r.get("created_by"),
            modify_at: r.get("modify_at"),
            modify_by: r.get("modify_by"),
        }))
    }

    async fn create(
        &self,
        req: CreatePurchaseRequest,
        dist: PurchaseDistributionResult,
        tx_date: NaiveDate,
        actor: &str,
    ) -> Result<PurchaseResponse, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Verify location
        let location_row = sqlx::query("SELECT id, name FROM locations WHERE id = $1")
            .bind(&req.location_id)
            .fetch_optional(&mut *tx)
            .await?;
        let loc = location_row.ok_or_else(|| {
            AppError::BadRequest(format!(
                "Lokasi dengan ID '{}' tidak ditemukan",
                req.location_id
            ))
        })?;
        let _location_name: String = loc.get("name");

        // 2. Verify supplier (if provided)
        let supplier_name = if let Some(sup_id) = &req.supplier_id {
            let sup_row = sqlx::query("SELECT id, name FROM suppliers WHERE id = $1")
                .bind(sup_id)
                .fetch_optional(&mut *tx)
                .await?;
            let s = sup_row.ok_or_else(|| {
                AppError::BadRequest(format!("Pemasok dengan ID '{sup_id}' tidak ditemukan"))
            })?;
            s.get::<String, _>("name")
        } else {
            req.supplier_name.clone().unwrap_or_default()
        };

        // 3. Verify Direct sessions: must exist and !is_locked && status != 'Completed'
        for (idx, item_req) in req.items.iter().enumerate() {
            let line_no = (idx + 1) as i32;
            if item_req.allocation_type == AllocationType::Direct {
                let sess_id = item_req.session_id.as_deref().ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "Baris #{line_no}: Alokasi biaya 'Direct' wajib memilih sesi tanam"
                    ))
                })?;

                let sess_row = sqlx::query(
                    "SELECT id, is_locked, status FROM planting_sessions WHERE id = $1 FOR UPDATE",
                )
                .bind(sess_id)
                .fetch_optional(&mut *tx)
                .await?;

                let s = sess_row.ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "Sesi tanam dengan ID '{sess_id}' tidak ditemukan"
                    ))
                })?;

                let is_locked: bool = s.get("is_locked");
                let status: String = s.get("status");

                if is_locked
                    || status.eq_ignore_ascii_case("Completed")
                    || status.eq_ignore_ascii_case("Cancelled")
                {
                    return Err(AppError::BadRequest(format!(
                        "Baris #{line_no}: Tidak dapat mengalokasikan biaya langsung ke sesi tanam yang sudah selesai, dibatalkan, atau terkunci"
                    )));
                }
            } else if item_req.session_id.is_some() {
                return Err(AppError::BadRequest(format!(
                    "Baris #{line_no}: Alokasi '{}' tidak boleh ditautkan ke sesi tanam",
                    item_req.allocation_type
                )));
            }
        }

        // 4. Generate invoice_number with transaction-scoped advisory lock
        let prefix = format!("PRC-{}", tx_date.format("%Y%m%d"));
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(&prefix)
            .execute(&mut *tx)
            .await?;

        let like_pattern = format!("{prefix}-%");
        let max_seq_row = sqlx::query(
            r#"
            SELECT COALESCE(MAX(CAST(SPLIT_PART(invoice_number, '-', 3) AS INTEGER)), 0) + 1 AS next_seq
            FROM purchase_headers
            WHERE invoice_number LIKE $1
            "#,
        )
        .bind(&like_pattern)
        .fetch_one(&mut *tx)
        .await?;

        let next_seq: i32 = max_seq_row.get("next_seq");
        let invoice_number = format_document_number("PRC", tx_date, next_seq);

        let purchase_id = format!("prc_{}", Uuid::new_v4());

        // 5. Insert purchase_headers
        sqlx::query(
            r#"
            INSERT INTO purchase_headers (
                id, invoice_number, transaction_date, supplier_id, supplier_name_snapshot,
                location_id, subtotal_amount, global_discount, global_shipping, total_amount,
                notes, created_by, modify_by
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $12
            )
            "#,
        )
        .bind(&purchase_id)
        .bind(&invoice_number)
        .bind(tx_date)
        .bind(&req.supplier_id)
        .bind(&supplier_name)
        .bind(&req.location_id)
        .bind(dist.subtotal_amount)
        .bind(dist.global_discount)
        .bind(dist.global_shipping)
        .bind(dist.total_amount)
        .bind(req.notes.as_deref().unwrap_or(""))
        .bind(actor)
        .execute(&mut *tx)
        .await?;

        // 6. Collect unique item_ids, sort ASC for anti-deadlock
        let mut distinct_item_ids: Vec<String> = req
            .items
            .iter()
            .filter_map(|it| it.item_id.clone())
            .collect();
        distinct_item_ids.sort();
        distinct_item_ids.dedup();

        let master_items_locked = if !distinct_item_ids.is_empty() {
            sqlx::query(
                r#"
                SELECT
                    m.id,
                    m.name,
                    m.default_unit,
                    m.current_stock,
                    m.average_unit_cost,
                    m.is_active,
                    COALESCE(ic.name, 'Umum') AS category_name
                FROM master_items m
                LEFT JOIN item_categories ic ON ic.id = m.category_id
                WHERE m.id = ANY($1)
                ORDER BY m.id ASC
                FOR UPDATE OF m
                "#,
            )
            .bind(&distinct_item_ids)
            .fetch_all(&mut *tx)
            .await?
        } else {
            Vec::new()
        };

        struct MasterItemState {
            name: String,
            default_unit: String,
            category: String,
            current_stock: Decimal,
            average_unit_cost: Decimal,
            is_active: bool,
        }

        let mut master_map: HashMap<String, MasterItemState> = HashMap::new();
        for row in master_items_locked {
            let mid: String = row.get("id");
            master_map.insert(
                mid,
                MasterItemState {
                    name: row.get("name"),
                    default_unit: row.get("default_unit"),
                    category: row.get("category_name"),
                    current_stock: row.get("current_stock"),
                    average_unit_cost: row.get("average_unit_cost"),
                    is_active: row.get("is_active"),
                },
            );
        }

        // Verify all provided item_ids exist and are active
        for mid in &distinct_item_ids {
            let state = master_map.get(mid).ok_or_else(|| {
                AppError::BadRequest(format!("Katalog barang '{mid}' tidak ditemukan"))
            })?;
            if !state.is_active {
                return Err(AppError::BadRequest(format!(
                    "Barang '{}' sedang tidak aktif",
                    state.name
                )));
            }
        }

        // 7. Insert purchase_items and perform auto-sync STOCK_IN
        let mut created_item_responses = Vec::with_capacity(req.items.len());

        for (idx, item_req) in req.items.iter().enumerate() {
            let pit_id = format!("pit_{}", Uuid::new_v4());
            let line_dist = &dist.items[idx];
            let is_synced = item_req.item_id.is_some();

            let (final_item_name, final_unit, final_category) = if let Some(mid) = &item_req.item_id
            {
                let m = master_map.get(mid).unwrap();
                let name = if item_req.item_name.trim().is_empty() {
                    m.name.clone()
                } else {
                    item_req.item_name.trim().to_string()
                };
                let unit = item_req
                    .unit
                    .as_deref()
                    .unwrap_or(&m.default_unit)
                    .trim()
                    .to_uppercase();
                let cat = item_req
                    .category
                    .as_deref()
                    .unwrap_or(&m.category)
                    .trim()
                    .to_string();
                (name, unit, cat)
            } else {
                let trimmed_name = item_req.item_name.trim().to_string();
                if trimmed_name.is_empty() {
                    return Err(AppError::BadRequest(format!(
                        "Baris #{}: Nama barang wajib diisi",
                        idx + 1
                    )));
                }
                let unit = item_req
                    .unit
                    .as_deref()
                    .unwrap_or("PCS")
                    .trim()
                    .to_uppercase();
                let cat = item_req
                    .category
                    .as_deref()
                    .unwrap_or("Umum")
                    .trim()
                    .to_string();
                (trimmed_name, unit, cat)
            };

            sqlx::query(
                r#"
                INSERT INTO purchase_items (
                    id, purchase_id, line_number, item_id, item_name, category, unit,
                    quantity, price_per_unit, subtotal, discount, shipping_cost, total_cost,
                    effective_unit_cost, allocation_type, session_id, allocation_days,
                    daily_allocation_rate, is_synced_to_inventory
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, $12, $13,
                    $14, $15, $16, $17,
                    $18, $19
                )
                "#,
            )
            .bind(&pit_id)
            .bind(&purchase_id)
            .bind(line_dist.line_number)
            .bind(&item_req.item_id)
            .bind(&final_item_name)
            .bind(&final_category)
            .bind(&final_unit)
            .bind(item_req.quantity)
            .bind(item_req.price_per_unit)
            .bind(line_dist.subtotal)
            .bind(line_dist.discount)
            .bind(line_dist.shipping_cost)
            .bind(line_dist.total_cost)
            .bind(line_dist.effective_unit_cost)
            .bind(item_req.allocation_type.to_string())
            .bind(&item_req.session_id)
            .bind(line_dist.allocation_days)
            .bind(line_dist.daily_allocation_rate)
            .bind(is_synced)
            .execute(&mut *tx)
            .await?;

            // If synced, record STOCK_IN in inventory_ledgers and update master_items
            if let Some(mid) = &item_req.item_id {
                let m = master_map.get_mut(mid).unwrap();
                let calc = calculate_stock_movement(
                    m.current_stock,
                    m.average_unit_cost,
                    MovementType::StockIn,
                    item_req.quantity,
                    Some(line_dist.effective_unit_cost),
                )?;

                let ledger_id = format!("rec_{}", Uuid::new_v4());
                let ledger_notes = format!("Pengadaan Nota {invoice_number}");

                sqlx::query(
                    r#"
                    INSERT INTO inventory_ledgers (
                        id, item_id, supplier_id, movement_type, quantity, delta_quantity,
                        balance_before, balance_after, unit_cost, total_cost_value,
                        average_unit_cost_after, reference_no, notes, purchase_item_id, created_by
                    ) VALUES (
                        $1, $2, $3, 'STOCK_IN', $4, $5,
                        $6, $7, $8, $9,
                        $10, $11, $12, $13, $14
                    )
                    "#,
                )
                .bind(&ledger_id)
                .bind(mid)
                .bind(&req.supplier_id)
                .bind(calc.quantity)
                .bind(calc.delta_quantity)
                .bind(calc.balance_before)
                .bind(calc.balance_after)
                .bind(calc.unit_cost)
                .bind(calc.total_cost_value)
                .bind(calc.average_unit_cost_after)
                .bind(&invoice_number)
                .bind(&ledger_notes)
                .bind(&pit_id)
                .bind(actor)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    r#"
                    UPDATE master_items
                    SET current_stock = $1, average_unit_cost = $2, modify_at = CURRENT_TIMESTAMP, modify_by = $3
                    WHERE id = $4
                    "#,
                )
                .bind(calc.balance_after)
                .bind(calc.average_unit_cost_after)
                .bind(actor)
                .bind(mid)
                .execute(&mut *tx)
                .await?;

                m.current_stock = calc.balance_after;
                m.average_unit_cost = calc.average_unit_cost_after;
            }

            created_item_responses.push(PurchaseItemResponse {
                id: pit_id,
                purchase_id: purchase_id.clone(),
                line_number: line_dist.line_number,
                item_id: item_req.item_id.clone(),
                item_name: final_item_name,
                category: final_category,
                unit: final_unit,
                quantity: item_req.quantity,
                price_per_unit: item_req.price_per_unit,
                subtotal: line_dist.subtotal,
                discount: line_dist.discount,
                shipping_cost: line_dist.shipping_cost,
                total_cost: line_dist.total_cost,
                effective_unit_cost: line_dist.effective_unit_cost,
                allocation_type: item_req.allocation_type,
                session_id: item_req.session_id.clone(),
                session_name: None, // Will be populated in fetch or detail
                allocation_days: line_dist.allocation_days,
                daily_allocation_rate: line_dist.daily_allocation_rate,
                is_synced_to_inventory: is_synced,
            });
        }

        tx.commit().await?;

        // Re-read or return constructed response
        let res = self.find_by_id(&purchase_id).await?.ok_or_else(|| {
            AppError::Internal("Gagal memuat nota pembelian yang baru dibuat".into())
        })?;

        Ok(res)
    }

    async fn delete(&self, id: &str, actor: &str) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Lock header
        let header_row =
            sqlx::query("SELECT id, invoice_number FROM purchase_headers WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;

        let h = header_row
            .ok_or_else(|| AppError::NotFound(format!("Purchase with ID '{id}' not found")))?;

        let invoice_number: String = h.get("invoice_number");

        // 2. Fetch items
        let items = sqlx::query(
            r#"
            SELECT
                id, item_id, item_name, quantity, allocation_type, session_id, is_synced_to_inventory
            FROM purchase_items
            WHERE purchase_id = $1
            "#,
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        // 3. Immutability guard: if Direct session is locked, reject
        for row in &items {
            let alloc_str: String = row.get("allocation_type");
            if alloc_str == "Direct" {
                if let Some(sess_id) = row.get::<Option<String>, _>("session_id") {
                    let s_row = sqlx::query(
                        "SELECT is_locked, status FROM planting_sessions WHERE id = $1",
                    )
                    .bind(&sess_id)
                    .fetch_optional(&mut *tx)
                    .await?;
                    if let Some(s) = s_row {
                        let is_locked: bool = s.get("is_locked");
                        let status: String = s.get("status");
                        if is_locked
                            || status.eq_ignore_ascii_case("Completed")
                            || status.eq_ignore_ascii_case("Cancelled")
                        {
                            return Err(AppError::BadRequest(
                                "Nota pembelian tidak dapat dihapus karena terikat pada sesi tanam yang sudah selesai, dibatalkan, atau terkunci".into(),
                            ));
                        }
                    }
                }
            }
        }

        // 4. Calculate total reversal quantities per synced item
        let mut reversal_map: HashMap<String, (String, Decimal)> = HashMap::new();
        for row in &items {
            let is_synced: bool = row.get("is_synced_to_inventory");
            if is_synced {
                if let Some(mid) = row.get::<Option<String>, _>("item_id") {
                    let item_name: String = row.get("item_name");
                    let qty: Decimal = row.get("quantity");
                    reversal_map
                        .entry(mid)
                        .and_modify(|e| e.1 += qty)
                        .or_insert((item_name, qty));
                }
            }
        }

        if !reversal_map.is_empty() {
            let mut distinct_item_ids: Vec<String> = reversal_map.keys().cloned().collect();
            distinct_item_ids.sort();

            let master_rows = sqlx::query(
                r#"
                SELECT id, name, current_stock, average_unit_cost
                FROM master_items
                WHERE id = ANY($1)
                ORDER BY id ASC
                FOR UPDATE
                "#,
            )
            .bind(&distinct_item_ids)
            .fetch_all(&mut *tx)
            .await?;

            for m_row in master_rows {
                let mid: String = m_row.get("id");
                let m_name: String = m_row.get("name");
                let current_stock: Decimal = m_row.get("current_stock");
                let avg_cost: Decimal = m_row.get("average_unit_cost");

                let required_reversal =
                    reversal_map.get(&mid).map(|e| e.1).unwrap_or(Decimal::ZERO);

                if current_stock < required_reversal {
                    return Err(AppError::BadRequest(format!(
                        "Nota pembelian tidak dapat dihapus karena stok barang '{m_name}' sudah terpakai (tersedia: {current_stock}, perlu dibatalkan: {required_reversal})"
                    )));
                }

                // Insert compensating RETURN_OUT ledger
                let calc = calculate_stock_movement(
                    current_stock,
                    avg_cost,
                    MovementType::ReturnOut,
                    required_reversal,
                    None,
                )?;

                let ledger_id = format!("rec_{}", Uuid::new_v4());
                let rev_ref = format!("REV-{invoice_number}");
                let rev_notes = format!("Pembatalan Nota {invoice_number}");

                sqlx::query(
                    r#"
                    INSERT INTO inventory_ledgers (
                        id, item_id, movement_type, quantity, delta_quantity,
                        balance_before, balance_after, unit_cost, total_cost_value,
                        average_unit_cost_after, reference_no, notes, purchase_item_id, created_by
                    ) VALUES (
                        $1, $2, 'RETURN_OUT', $3, $4,
                        $5, $6, $7, $8,
                        $9, $10, $11, NULL, $12
                    )
                    "#,
                )
                .bind(&ledger_id)
                .bind(&mid)
                .bind(calc.quantity)
                .bind(calc.delta_quantity)
                .bind(calc.balance_before)
                .bind(calc.balance_after)
                .bind(calc.unit_cost)
                .bind(calc.total_cost_value)
                .bind(calc.average_unit_cost_after)
                .bind(&rev_ref)
                .bind(&rev_notes)
                .bind(actor)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    r#"
                    UPDATE master_items
                    SET current_stock = $1, modify_at = CURRENT_TIMESTAMP, modify_by = $2
                    WHERE id = $3
                    "#,
                )
                .bind(calc.balance_after)
                .bind(actor)
                .bind(&mid)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 5. Delete purchase_headers (Cascades to purchase_items, sets NULL on old ledger purchase_item_id)
        sqlx::query("DELETE FROM purchase_headers WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
