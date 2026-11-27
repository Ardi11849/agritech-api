use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;
use crate::modules::finance::hpp::dto::{
    HppListQuery, HppListResponse, HppPurchaseCalcInput, HppSaleCalcInput, HppSessionCalcInput,
    HppSessionItem, HppSnapshotInfo, HppSummaryKpi, RHPPAllocatedPurchaseItem, RHPPDetailResponse,
    RHPPDirectPurchaseItem, RHPPRealizedSaleItem,
};
use crate::modules::finance::hpp::repository::HppRepository;

pub fn resolve_session_end_date(
    start_date: NaiveDate,
    harvest_date: Option<NaiveDate>,
    today: NaiveDate,
) -> NaiveDate {
    let effective = harvest_date.unwrap_or(today);
    std::cmp::max(start_date, effective)
}

pub fn build_location_daily_holes(
    sessions: &[HppSessionCalcInput],
    today: NaiveDate,
) -> HashMap<(String, NaiveDate), (i64, usize)> {
    let mut map = HashMap::new();
    for s in sessions {
        if s.status.eq_ignore_ascii_case("Cancelled") {
            continue;
        }
        let end_date = resolve_session_end_date(s.start_date, s.harvest_date, today);
        let mut curr = s.start_date;
        while curr <= end_date {
            let entry = map
                .entry((s.location_id.clone(), curr))
                .or_insert((0i64, 0usize));
            entry.0 += s.total_holes as i64;
            entry.1 += 1;
            curr += chrono::Duration::days(1);
        }
    }
    map
}

#[derive(Debug, Clone)]
pub struct RHPPComputation {
    pub summary: HppSessionItem,
    pub snapshot: HppSnapshotInfo,
    pub direct_purchase_items: Vec<RHPPDirectPurchaseItem>,
    pub allocated_purchase_items: Vec<RHPPAllocatedPurchaseItem>,
    pub realized_sales: Vec<RHPPRealizedSaleItem>,
}

pub fn calculate_session_rhpp(
    target: &HppSessionCalcInput,
    purchases: &[HppPurchaseCalcInput],
    sales: &[HppSaleCalcInput],
    today: NaiveDate,
    daily_holes: &HashMap<(String, NaiveDate), (i64, usize)>,
) -> RHPPComputation {
    let end_date = resolve_session_end_date(target.start_date, target.harvest_date, today);
    let duration_days = (end_date - target.start_date).num_days() + 1;

    // 1. Direct purchases
    let mut direct_purchase_items = Vec::new();
    let mut direct_costs = Decimal::ZERO;

    for p in purchases {
        if p.allocation_type.eq_ignore_ascii_case("Direct")
            && p.session_id.as_deref() == Some(&target.session_id)
        {
            direct_costs += p.total_cost;
            direct_purchase_items.push(RHPPDirectPurchaseItem {
                purchase_item_id: p.purchase_item_id.clone(),
                purchase_id: p.purchase_id.clone(),
                invoice_number: p.invoice_number.clone(),
                transaction_date: p.transaction_date,
                item_name: p.item_name.clone(),
                category: p.category.clone(),
                unit: p.unit.clone(),
                quantity: p.quantity,
                total_cost: p.total_cost,
                allocation_type: "Direct".into(),
            });
        }
    }
    direct_costs = round_to_two_decimals(direct_costs);

    // 2. Allocated purchases (Shared & Asset)
    let mut allocated_purchase_items = Vec::new();
    let mut allocated_shared_costs = Decimal::ZERO;
    let mut allocated_asset_costs = Decimal::ZERO;

    for p in purchases {
        let is_shared = p.allocation_type.eq_ignore_ascii_case("Shared");
        let is_asset = p.allocation_type.eq_ignore_ascii_case("Asset");

        if (is_shared || is_asset) && p.location_id == target.location_id && p.allocation_days >= 1
        {
            let p_start = p.transaction_date;
            let p_end = p_start + chrono::Duration::days((p.allocation_days - 1) as i64);

            let i_start = std::cmp::max(target.start_date, p_start);
            let i_end = std::cmp::min(end_date, p_end);

            if i_start <= i_end {
                let overlap_days = (i_end - i_start).num_days() + 1;
                let daily_rate = p.total_cost / Decimal::from(p.allocation_days);

                let mut sum_ratio = Decimal::ZERO;
                let mut sum_allocated_raw = Decimal::ZERO;

                let mut curr = i_start;
                while curr <= i_end {
                    let (h_total, active_count) = daily_holes
                        .get(&(target.location_id.clone(), curr))
                        .cloned()
                        .unwrap_or((0, 0));

                    let ratio = if h_total > 0 {
                        Decimal::from(target.total_holes) / Decimal::from(h_total)
                    } else if active_count > 0 {
                        Decimal::ONE / Decimal::from(active_count)
                    } else {
                        Decimal::ONE
                    };

                    sum_ratio += ratio;
                    sum_allocated_raw += daily_rate * ratio;
                    curr += chrono::Duration::days(1);
                }

                let allocated_cost = round_to_two_decimals(sum_allocated_raw);
                let avg_ratio_percent = if overlap_days > 0 {
                    round_to_two_decimals(
                        (sum_ratio / Decimal::from(overlap_days)) * Decimal::from(100),
                    )
                } else {
                    Decimal::ZERO
                };

                let alloc_type = if is_shared { "Shared" } else { "Asset" };

                if is_shared {
                    allocated_shared_costs += allocated_cost;
                } else {
                    allocated_asset_costs += allocated_cost;
                }

                allocated_purchase_items.push(RHPPAllocatedPurchaseItem {
                    purchase_item_id: p.purchase_item_id.clone(),
                    purchase_id: p.purchase_id.clone(),
                    invoice_number: p.invoice_number.clone(),
                    transaction_date: p.transaction_date,
                    item_name: p.item_name.clone(),
                    category: p.category.clone(),
                    allocation_type: alloc_type.into(),
                    item_total_cost: p.total_cost,
                    allocation_days: p.allocation_days,
                    daily_rate: round_to_two_decimals(daily_rate),
                    overlap_days,
                    overlap_start_date: i_start,
                    overlap_end_date: i_end,
                    average_ratio_percent: avg_ratio_percent,
                    allocated_cost,
                });
            }
        }
    }
    allocated_shared_costs = round_to_two_decimals(allocated_shared_costs);
    allocated_asset_costs = round_to_two_decimals(allocated_asset_costs);
    let allocated_costs = round_to_two_decimals(allocated_shared_costs + allocated_asset_costs);
    let computed_total_hpp = round_to_two_decimals(direct_costs + allocated_costs);

    // 3. Realized Sales
    let mut realized_sales = Vec::new();
    let mut computed_revenue = Decimal::ZERO;
    let mut computed_subsidy = Decimal::ZERO;
    let mut sum_sales_qty = Decimal::ZERO;

    for s in sales {
        if s.session_id == target.session_id {
            computed_revenue += s.total_revenue;
            computed_subsidy += s.seller_shipping_subsidy;
            sum_sales_qty += s.quantity;

            realized_sales.push(RHPPRealizedSaleItem {
                sale_id: s.sale_id.clone(),
                sale_number: s.sale_number.clone(),
                sale_date: s.sale_date,
                customer_name: s.customer_name.clone(),
                unit: s.unit.clone(),
                quantity: s.quantity,
                price_per_unit: s.price_per_unit,
                gross_revenue: s.gross_revenue,
                discount: s.discount,
                total_revenue: s.total_revenue,
                seller_shipping_subsidy: s.seller_shipping_subsidy,
                net_proceeds: s.net_proceeds,
                payment_status: s.payment_status.clone(),
            });
        }
    }
    computed_revenue = round_to_two_decimals(computed_revenue);
    computed_subsidy = round_to_two_decimals(computed_subsidy);

    // 4. Immutability / Snapshot guard
    let is_snapshot_locked = target.hpp_snapshot_locked_at.is_some();
    let total_hpp = if is_snapshot_locked {
        target.final_hpp.unwrap_or(computed_total_hpp)
    } else {
        computed_total_hpp
    };
    let total_revenue = if is_snapshot_locked {
        target.final_revenue.unwrap_or(computed_revenue)
    } else {
        computed_revenue
    };
    let total_shipping_subsidy = if is_snapshot_locked {
        target.final_shipping_subsidy.unwrap_or(computed_subsidy)
    } else {
        computed_subsidy
    };

    let net_revenue = round_to_two_decimals(total_revenue - total_shipping_subsidy);
    let net_profit = if is_snapshot_locked {
        target.final_net_profit.unwrap_or(net_revenue - total_hpp)
    } else {
        round_to_two_decimals(net_revenue - total_hpp)
    };

    let profit_margin_percent = if total_revenue > Decimal::ZERO {
        round_to_two_decimals((net_profit / total_revenue) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    let effective_yield_kg = if target.actual_yield_kg > Decimal::ZERO {
        target.actual_yield_kg
    } else {
        round_to_two_decimals(sum_sales_qty)
    };

    let cost_per_kg = if effective_yield_kg > Decimal::ZERO {
        round_to_two_decimals(total_hpp / effective_yield_kg)
    } else {
        Decimal::ZERO
    };

    let net_revenue_per_kg = if effective_yield_kg > Decimal::ZERO {
        round_to_two_decimals(net_revenue / effective_yield_kg)
    } else {
        Decimal::ZERO
    };

    let profit_per_kg = round_to_two_decimals(net_revenue_per_kg - cost_per_kg);

    let waste_percentage = if target.total_holes > 0 {
        round_to_two_decimals(
            (Decimal::from(target.mortality_count) / Decimal::from(target.total_holes))
                * Decimal::from(100),
        )
    } else {
        Decimal::ZERO
    };

    let summary = HppSessionItem {
        session_id: target.session_id.clone(),
        session_name: target.session_name.clone(),
        plant_name: target.plant_name.clone(),
        plant_variety: target.plant_variety.clone(),
        location_id: target.location_id.clone(),
        location_name: target.location_name.clone(),
        status: target.status.clone(),
        start_date: target.start_date,
        end_date,
        harvest_date: target.harvest_date,
        duration_days,
        total_holes: target.total_holes,
        waste_qty: target.mortality_count,
        waste_percentage,
        direct_costs,
        allocated_shared_costs,
        allocated_asset_costs,
        allocated_costs,
        total_hpp,
        total_revenue,
        total_shipping_subsidy,
        net_revenue,
        net_profit,
        profit_margin_percent,
        effective_yield_kg,
        cost_per_kg,
        net_revenue_per_kg,
        profit_per_kg,
        is_snapshot_locked,
        hpp_snapshot_locked_at: target.hpp_snapshot_locked_at,
    };

    let snapshot = HppSnapshotInfo {
        final_hpp: target.final_hpp,
        final_revenue: target.final_revenue,
        final_shipping_subsidy: target.final_shipping_subsidy,
        final_net_profit: target.final_net_profit,
        locked_at: target.hpp_snapshot_locked_at,
        locked_by: target.hpp_snapshot_locked_by.clone(),
    };

    RHPPComputation {
        summary,
        snapshot,
        direct_purchase_items,
        allocated_purchase_items,
        realized_sales,
    }
}

pub fn calculate_hpp_kpi(items: &[HppSessionItem]) -> HppSummaryKpi {
    let mut total_cumulative_hpp = Decimal::ZERO;
    let mut total_cumulative_revenue = Decimal::ZERO;
    let mut total_cumulative_shipping_subsidy = Decimal::ZERO;
    let mut total_cumulative_net_profit = Decimal::ZERO;
    let mut profitable_sessions_count = 0i64;
    let mut loss_sessions_count = 0i64;

    for item in items {
        total_cumulative_hpp += item.total_hpp;
        total_cumulative_revenue += item.total_revenue;
        total_cumulative_shipping_subsidy += item.total_shipping_subsidy;
        total_cumulative_net_profit += item.net_profit;

        if item.net_profit > Decimal::ZERO {
            profitable_sessions_count += 1;
        } else if item.net_profit < Decimal::ZERO {
            loss_sessions_count += 1;
        }
    }

    let average_profit_margin_percent = if total_cumulative_revenue > Decimal::ZERO {
        round_to_two_decimals(
            (total_cumulative_net_profit / total_cumulative_revenue) * Decimal::from(100),
        )
    } else {
        Decimal::ZERO
    };

    HppSummaryKpi {
        total_cumulative_hpp,
        total_cumulative_revenue,
        total_cumulative_shipping_subsidy,
        total_cumulative_net_profit,
        average_profit_margin_percent,
        profitable_sessions_count,
        loss_sessions_count,
    }
}

#[async_trait]
pub trait HppService: Send + Sync {
    async fn get_hpp_list(&self, query: &HppListQuery) -> Result<HppListResponse, AppError>;
    async fn get_session_rhpp(
        &self,
        session_id: &str,
        eval_date: Option<NaiveDate>,
    ) -> Result<RHPPDetailResponse, AppError>;
    async fn lock_session_snapshot(
        &self,
        session_id: &str,
        actor: &str,
    ) -> Result<RHPPDetailResponse, AppError>;
}

pub struct HppServiceImpl {
    repo: Arc<dyn HppRepository>,
}

impl HppServiceImpl {
    pub fn new(repo: Arc<dyn HppRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl HppService for HppServiceImpl {
    async fn get_hpp_list(&self, query: &HppListQuery) -> Result<HppListResponse, AppError> {
        // Validate status enum if provided
        if let Some(status) = &query.status {
            let s = status.trim();
            if !s.is_empty()
                && !s.eq_ignore_ascii_case("Active")
                && !s.eq_ignore_ascii_case("Completed")
            {
                return Err(AppError::BadRequest(format!(
                    "Filter status '{s}' tidak valid. Nilai yang diizinkan: 'Active' atau 'Completed'"
                )));
            }
        }

        let today = query.date.unwrap_or_else(today_wib);

        let sessions = self.repo.find_all_sessions().await?;
        let purchases = self.repo.find_all_purchases().await?;
        let sales = self.repo.find_all_sales().await?;

        let daily_holes = build_location_daily_holes(&sessions, today);

        let mut computed_items = Vec::with_capacity(sessions.len());
        for s in &sessions {
            let comp = calculate_session_rhpp(s, &purchases, &sales, today, &daily_holes);
            computed_items.push(comp.summary);
        }

        // Apply filters
        let mut filtered: Vec<HppSessionItem> = computed_items
            .into_iter()
            .filter(|item| {
                if let Some(loc_id) = &query.location_id {
                    if !loc_id.trim().is_empty() && item.location_id != *loc_id {
                        return false;
                    }
                }
                if let Some(status) = &query.status {
                    if !status.trim().is_empty() && !item.status.eq_ignore_ascii_case(status.trim())
                    {
                        return false;
                    }
                }
                if let Some(search) = &query.search {
                    let term = search.trim().to_lowercase();
                    if !term.is_empty() {
                        let matched = item.session_name.to_lowercase().contains(&term)
                            || item.plant_name.to_lowercase().contains(&term)
                            || item.plant_variety.to_lowercase().contains(&term)
                            || item.location_name.to_lowercase().contains(&term);
                        if !matched {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();

        // Sort
        let sort_asc = query
            .sort_order
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("asc"))
            .unwrap_or(false);

        if sort_asc {
            filtered.sort_by_key(|a| a.start_date);
        } else {
            filtered.sort_by_key(|a| std::cmp::Reverse(a.start_date));
        }

        let kpi = calculate_hpp_kpi(&filtered);

        Ok(HppListResponse {
            kpi,
            items: filtered,
        })
    }

    async fn get_session_rhpp(
        &self,
        session_id: &str,
        eval_date: Option<NaiveDate>,
    ) -> Result<RHPPDetailResponse, AppError> {
        let today = eval_date.unwrap_or_else(today_wib);

        let target_raw = self.repo.find_session_by_id(session_id).await?;
        let target = match target_raw {
            Some(t) => {
                if t.status.eq_ignore_ascii_case("Cancelled") {
                    return Err(AppError::BadRequest(format!(
                        "Sesi tanam dengan ID '{session_id}' telah dibatalkan (Cancelled) dan tidak dapat dianalisa"
                    )));
                }
                t
            }
            None => {
                return Err(AppError::NotFound(format!(
                    "Sesi tanam dengan ID '{session_id}' tidak ditemukan"
                )));
            }
        };

        let sessions = self.repo.find_all_sessions().await?;
        let purchases = self.repo.find_all_purchases().await?;
        let sales = self.repo.find_all_sales().await?;
        let daily_holes = build_location_daily_holes(&sessions, today);

        let comp = calculate_session_rhpp(&target, &purchases, &sales, today, &daily_holes);

        Ok(RHPPDetailResponse {
            summary: comp.summary,
            snapshot: comp.snapshot,
            direct_purchase_items: comp.direct_purchase_items,
            allocated_purchase_items: comp.allocated_purchase_items,
            realized_sales: comp.realized_sales,
        })
    }

    async fn lock_session_snapshot(
        &self,
        session_id: &str,
        actor: &str,
    ) -> Result<RHPPDetailResponse, AppError> {
        let today = today_wib();
        let target_raw = self.repo.find_session_by_id(session_id).await?;
        let target = match target_raw {
            Some(t) => {
                if t.status.eq_ignore_ascii_case("Cancelled") {
                    return Err(AppError::BadRequest(format!(
                        "Sesi tanam dengan ID '{session_id}' telah dibatalkan (Cancelled) dan tidak dapat dikunci"
                    )));
                }
                t
            }
            None => {
                return Err(AppError::NotFound(format!(
                    "Sesi tanam dengan ID '{session_id}' tidak ditemukan"
                )));
            }
        };

        if !target.status.eq_ignore_ascii_case("Completed") {
            return Err(AppError::BadRequest(
                "Hanya sesi tanam berstatus 'Completed' yang dapat dikunci snapshot HPP-nya".into(),
            ));
        }

        if target.hpp_snapshot_locked_at.is_some() {
            return Err(AppError::BadRequest(
                "Snapshot HPP untuk sesi ini sudah pernah dikunci secara permanen".into(),
            ));
        }

        let sessions = self.repo.find_all_sessions().await?;
        let purchases = self.repo.find_all_purchases().await?;
        let sales = self.repo.find_all_sales().await?;
        let daily_holes = build_location_daily_holes(&sessions, today);

        let comp = calculate_session_rhpp(&target, &purchases, &sales, today, &daily_holes);

        self.repo
            .lock_snapshot(
                session_id,
                comp.summary.total_hpp,
                comp.summary.total_revenue,
                comp.summary.total_shipping_subsidy,
                comp.summary.net_profit,
                actor,
            )
            .await?;

        // Return updated detail
        self.get_session_rhpp(session_id, Some(today)).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_hpp_simulasi_1_prorata_and_unit_economics() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();

        // 2 Concurrent sessions in location "Greenhouse Lembang"
        let session_a = HppSessionCalcInput {
            session_id: "ses_A".into(),
            session_name: "Sesi A - Selada".into(),
            plant_name: "Selada Romaine".into(),
            plant_variety: "Green Towers".into(),
            location_id: "loc_lembang".into(),
            location_name: "Greenhouse Lembang".into(),
            status: "Completed".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
            harvest_date: Some(NaiveDate::from_ymd_opt(2026, 11, 10).unwrap()),
            total_holes: 600,
            mortality_count: 30,
            actual_yield_kg: d("100.00"),
            final_hpp: None,
            final_revenue: None,
            final_shipping_subsidy: None,
            final_net_profit: None,
            hpp_snapshot_locked_at: None,
            hpp_snapshot_locked_by: None,
        };

        let session_b = HppSessionCalcInput {
            session_id: "ses_B".into(),
            session_name: "Sesi B - Pakcoy".into(),
            plant_name: "Pakcoy".into(),
            plant_variety: "Nauli F1".into(),
            location_id: "loc_lembang".into(),
            location_name: "Greenhouse Lembang".into(),
            status: "Active".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
            harvest_date: None,
            total_holes: 400,
            mortality_count: 10,
            actual_yield_kg: d("0.00"),
            final_hpp: None,
            final_revenue: None,
            final_shipping_subsidy: None,
            final_net_profit: None,
            hpp_snapshot_locked_at: None,
            hpp_snapshot_locked_by: None,
        };

        let sessions = vec![session_a.clone(), session_b];
        let daily_holes = build_location_daily_holes(&sessions, today);

        // Check total holes daily on day 1
        let day1 = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let (h_total, count) = daily_holes.get(&("loc_lembang".into(), day1)).unwrap();
        assert_eq!(*h_total, 1000);
        assert_eq!(*count, 2);

        // Purchases:
        // 1. Direct for Sesi A: 500,000.00
        // 2. Shared: 1,000,000.00, 10 days (DailyRate = 100,000)
        // 3. Asset: 7,300,000.00, 730 days (DailyRate = 10,000)
        let purchases = vec![
            HppPurchaseCalcInput {
                purchase_item_id: "pit_1".into(),
                purchase_id: "prc_1".into(),
                invoice_number: "PRC-20261101-001".into(),
                transaction_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
                location_id: "loc_lembang".into(),
                item_name: "Benih & Nutrisi Langsung".into(),
                category: "Nutrisi & Pupuk".into(),
                unit: "PACK".into(),
                quantity: d("10.00"),
                total_cost: d("500000.00"),
                allocation_type: "Direct".into(),
                allocation_days: 1,
                session_id: Some("ses_A".into()),
            },
            HppPurchaseCalcInput {
                purchase_item_id: "pit_2".into(),
                purchase_id: "prc_2".into(),
                invoice_number: "PRC-20261101-002".into(),
                transaction_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
                location_id: "loc_lembang".into(),
                item_name: "Listrik & Nutrisi Umum".into(),
                category: "Utilitas".into(),
                unit: "UNIT".into(),
                quantity: d("1.00"),
                total_cost: d("1000000.00"),
                allocation_type: "Shared".into(),
                allocation_days: 10,
                session_id: None,
            },
            HppPurchaseCalcInput {
                purchase_item_id: "pit_3".into(),
                purchase_id: "prc_3".into(),
                invoice_number: "PRC-20261101-003".into(),
                transaction_date: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
                location_id: "loc_lembang".into(),
                item_name: "Pompa Tekanan Tinggi".into(),
                category: "Peralatan & Aset".into(),
                unit: "UNIT".into(),
                quantity: d("1.00"),
                total_cost: d("7300000.00"),
                allocation_type: "Asset".into(),
                allocation_days: 730,
                session_id: None,
            },
        ];

        // Sales for Sesi A: 3,000,000 revenue, 100,000 subsidy
        let sales = vec![HppSaleCalcInput {
            sale_id: "sls_1".into(),
            sale_number: "SLS-20261110-001".into(),
            sale_date: NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(),
            session_id: "ses_A".into(),
            customer_name: "Restoran Healthy Bowl".into(),
            unit: "KG".into(),
            quantity: d("100.00"),
            price_per_unit: d("30000.00"),
            gross_revenue: d("3000000.00"),
            discount: d("0.00"),
            total_revenue: d("3000000.00"),
            seller_shipping_subsidy: d("100000.00"),
            net_proceeds: d("2900000.00"),
            payment_status: "Lunas".into(),
        }];

        let comp = calculate_session_rhpp(&session_a, &purchases, &sales, today, &daily_holes);
        let res = comp.summary;

        // Assertions matching PRD Simulasi 1 exactly
        assert_eq!(res.direct_costs, d("500000.00"));
        assert_eq!(res.allocated_shared_costs, d("600000.00"));
        assert_eq!(res.allocated_asset_costs, d("60000.00"));
        assert_eq!(res.allocated_costs, d("660000.00"));
        assert_eq!(res.total_hpp, d("1160000.00"));
        assert_eq!(res.total_revenue, d("3000000.00"));
        assert_eq!(res.total_shipping_subsidy, d("100000.00"));
        assert_eq!(res.net_revenue, d("2900000.00"));
        assert_eq!(res.net_profit, d("1740000.00"));
        assert_eq!(res.profit_margin_percent, d("58.00"));
        assert_eq!(res.cost_per_kg, d("11600.00"));
        assert_eq!(res.net_revenue_per_kg, d("29000.00"));
        assert_eq!(res.profit_per_kg, d("17400.00"));
        assert_eq!(res.waste_percentage, d("5.00"));
    }

    #[test]
    fn test_hpp_zero_yield_and_zero_revenue_safe_division() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        let session = HppSessionCalcInput {
            session_id: "ses_zero".into(),
            session_name: "Sesi Baru".into(),
            plant_name: "Selada".into(),
            plant_variety: "Green".into(),
            location_id: "loc_1".into(),
            location_name: "Gudang".into(),
            status: "Active".into(),
            start_date: today,
            harvest_date: None,
            total_holes: 100,
            mortality_count: 0,
            actual_yield_kg: Decimal::ZERO,
            final_hpp: None,
            final_revenue: None,
            final_shipping_subsidy: None,
            final_net_profit: None,
            hpp_snapshot_locked_at: None,
            hpp_snapshot_locked_by: None,
        };

        let daily_holes = build_location_daily_holes(std::slice::from_ref(&session), today);
        let comp = calculate_session_rhpp(&session, &[], &[], today, &daily_holes);
        let res = comp.summary;

        assert_eq!(res.cost_per_kg, Decimal::ZERO);
        assert_eq!(res.net_revenue_per_kg, Decimal::ZERO);
        assert_eq!(res.profit_per_kg, Decimal::ZERO);
        assert_eq!(res.profit_margin_percent, Decimal::ZERO);
    }
}
