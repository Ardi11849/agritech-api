use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;
use crate::modules::finance::hpp::dto::{HppListQuery, HppSessionItem};
use crate::modules::finance::hpp::service::HppService;
use crate::modules::reports::ai_audit::dto::{
    AiAuditInsight, AiAuditScanQuery, AiAuditScanResponse, AiAuditStats, AnomalyCategory,
    AnomalySeverity,
};
use crate::modules::reports::ai_audit::repository::{
    AiAuditRepository, InventoryAuditItemRow, PurchaseItemHistoryRow, SaleAuditRow,
};

// Pure detection functions

pub fn detect_purchase_price_spikes(
    purchases: &[PurchaseItemHistoryRow],
    scanned_at: DateTime<Utc>,
) -> Vec<AiAuditInsight> {
    let mut insights = Vec::new();

    // Group items by normalized item_name + unit
    let mut item_groups: HashMap<(String, String), Vec<&PurchaseItemHistoryRow>> = HashMap::new();
    for p in purchases {
        let key = (
            p.item_name.trim().to_lowercase(),
            p.unit.trim().to_lowercase(),
        );
        item_groups.entry(key).or_default().push(p);
    }

    for ((_name_norm, _unit_norm), mut rows) in item_groups {
        // Must have at least 2 purchases in the 90-day window
        if rows.len() < 2 {
            continue;
        }

        // rows are already ordered by transaction_date DESC, created_at DESC
        // Ensure deterministic order
        rows.sort_by(|a, b| {
            b.transaction_date
                .cmp(&a.transaction_date)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });

        let latest = rows[0];
        let previous_rows = &rows[1..];

        // Calculate average historical unit cost
        let mut sum_cost = Decimal::ZERO;
        let mut count = Decimal::ZERO;
        for prev in previous_rows {
            if prev.effective_unit_cost > Decimal::ZERO {
                sum_cost += prev.effective_unit_cost;
                count += Decimal::ONE;
            }
        }

        if count == Decimal::ZERO {
            continue;
        }

        let avg_hist_cost = round_to_two_decimals(sum_cost / count);
        if avg_hist_cost <= Decimal::ZERO {
            continue;
        }

        let diff = latest.effective_unit_cost - avg_hist_cost;
        if diff <= Decimal::ZERO {
            continue;
        }

        let spike_pct = round_to_two_decimals((diff / avg_hist_cost) * Decimal::from(100));

        // Threshold > 20%
        if spike_pct > Decimal::from(20) {
            let severity = if spike_pct > Decimal::from(50) {
                AnomalySeverity::Critical
            } else {
                AnomalySeverity::Warning
            };

            let impact_value = round_to_two_decimals(diff * latest.quantity);

            let title = format!("Lonjakan Harga Pembelian Bahan Baku (+{}%)", spike_pct);
            let message = format!(
                "Harga satuan efektif {} pada nota {} naik menjadi Rp {} (rata-rata 90 hari sebelumnya Rp {}).",
                latest.item_name, latest.invoice_number, latest.effective_unit_cost, avg_hist_cost
            );
            let suggestion = "Bandingkan penawaran dari pemasok alternatif di Master Pemasok atau negosiasikan kontrak pembelian curah.".to_string();

            insights.push(AiAuditInsight {
                id: format!("prc_spike_{}", latest.purchase_item_id),
                category: AnomalyCategory::PURCHASE,
                severity,
                title,
                message,
                suggestion,
                impact_value,
                related_id: latest.purchase_id.clone(),
                target_path: "/finance/purchases".to_string(),
                detected_at: scanned_at,
            });
        }
    }

    insights
}

pub fn detect_hpp_anomalies(
    sessions: &[HppSessionItem],
    scanned_at: DateTime<Utc>,
) -> Vec<AiAuditInsight> {
    let mut insights = Vec::new();

    // 2B. Net Loss Check
    for s in sessions {
        if s.status.eq_ignore_ascii_case("Cancelled") {
            continue;
        }

        if s.total_revenue > Decimal::ZERO && s.net_profit < Decimal::ZERO {
            insights.push(AiAuditInsight {
                id: format!("hpp_loss_{}", s.session_id),
                category: AnomalyCategory::HPP,
                severity: AnomalySeverity::Critical,
                title: format!("Sesi Tanam Mengalami Kerugian Bersih ({})", s.session_name),
                message: format!(
                    "Sesi {} mencatatkan pendapatan Rp {} namun beban HPP mencapai Rp {}, menghasilkan rugi bersih Rp {}.",
                    s.session_name, s.total_revenue, s.total_hpp, s.net_profit.abs()
                ),
                suggestion: "Tinjau kembali struktur biaya langsung dan alokasi biaya bersama pada sesi ini untuk mencegah kerugian berulang.".to_string(),
                impact_value: s.net_profit.abs(),
                related_id: s.session_id.clone(),
                target_path: format!("/finance/hpp/{}", s.session_id),
                detected_at: scanned_at,
            });
        }

        // 2C. Waste Percentage Check (> 15% and total_hpp > 0)
        if s.waste_percentage > Decimal::from(15) && s.total_hpp > Decimal::ZERO {
            let waste_impact =
                round_to_two_decimals(s.total_hpp * (s.waste_percentage / Decimal::from(100)));
            insights.push(AiAuditInsight {
                id: format!("hpp_waste_{}", s.session_id),
                category: AnomalyCategory::HPP,
                severity: AnomalySeverity::Warning,
                title: format!("Tingkat Mortalitas Tanaman Tinggi ({}%)", s.waste_percentage),
                message: format!(
                    "Sesi {} mengalami kematian {} lubang dari total {} lubang ({}%), melebihi batas toleransi 15%.",
                    s.session_name, s.waste_qty, s.total_holes, s.waste_percentage
                ),
                suggestion: "Audit parameter sensor air/nutrisi (PPM dan pH) pada masa semai/vegetatif untuk mengidentifikasi penyebab kematian bibit.".to_string(),
                impact_value: waste_impact,
                related_id: s.session_id.clone(),
                target_path: format!("/finance/hpp/{}", s.session_id),
                detected_at: scanned_at,
            });
        }
    }

    // 2A. HPP Benchmark Spike per (plant_name, plant_variety) group
    let mut plant_groups: HashMap<(String, String), Vec<&HppSessionItem>> = HashMap::new();
    for s in sessions {
        if s.status.eq_ignore_ascii_case("Cancelled") {
            continue;
        }
        let key = (
            s.plant_name.trim().to_lowercase(),
            s.plant_variety.trim().to_lowercase(),
        );
        plant_groups.entry(key).or_default().push(s);
    }

    for ((_p_name, _p_var), group) in plant_groups {
        if group.len() < 2 {
            continue;
        }

        let yield_sessions: Vec<&&HppSessionItem> = group
            .iter()
            .filter(|s| s.cost_per_kg > Decimal::ZERO)
            .collect();

        if yield_sessions.len() >= 2 {
            // Benchmark based on cost_per_kg
            let sum_cost: Decimal = yield_sessions.iter().map(|s| s.cost_per_kg).sum();
            let avg_cost = round_to_two_decimals(sum_cost / Decimal::from(yield_sessions.len()));

            if avg_cost > Decimal::ZERO {
                for s in yield_sessions {
                    let diff = s.cost_per_kg - avg_cost;
                    if diff > Decimal::ZERO {
                        let pct = round_to_two_decimals((diff / avg_cost) * Decimal::from(100));
                        if pct > Decimal::from(25) {
                            let impact = round_to_two_decimals(diff * s.effective_yield_kg);
                            insights.push(AiAuditInsight {
                                id: format!("hpp_spike_{}", s.session_id),
                                category: AnomalyCategory::HPP,
                                severity: AnomalySeverity::Critical,
                                title: format!("Pembengkakan HPP per Kg (+{}%)", pct),
                                message: format!(
                                    "HPP per Kg untuk sesi {} mencapai Rp {}/Kg, lebih tinggi {}% dari rata-rata varietas serupa (Rp {}/Kg).",
                                    s.session_name, s.cost_per_kg, pct, avg_cost
                                ),
                                suggestion: "Periksa efisiensi penggunaan nutrisi dan alokasi aset amortisasi pada lokasi instalasi terkait.".to_string(),
                                impact_value: impact,
                                related_id: s.session_id.clone(),
                                target_path: format!("/finance/hpp/{}", s.session_id),
                                detected_at: scanned_at,
                            });
                        }
                    }
                }
            }
        } else {
            // Benchmark based on total_hpp
            let sum_hpp: Decimal = group.iter().map(|s| s.total_hpp).sum();
            let avg_hpp = round_to_two_decimals(sum_hpp / Decimal::from(group.len()));

            if avg_hpp > Decimal::ZERO {
                for s in &group {
                    let diff = s.total_hpp - avg_hpp;
                    if diff > Decimal::ZERO {
                        let pct = round_to_two_decimals((diff / avg_hpp) * Decimal::from(100));
                        if pct > Decimal::from(25) {
                            let impact = round_to_two_decimals(diff);
                            insights.push(AiAuditInsight {
                                id: format!("hpp_spike_{}", s.session_id),
                                category: AnomalyCategory::HPP,
                                severity: AnomalySeverity::Critical,
                                title: format!("Pembengkakan Total Biaya HPP (+{}%)", pct),
                                message: format!(
                                    "Total biaya HPP untuk sesi {} mencapai Rp {}, lebih tinggi {}% dari rata-rata kelompok varietas (Rp {}).",
                                    s.session_name, s.total_hpp, pct, avg_hpp
                                ),
                                suggestion: "Periksa belanja sarana langsung dan durasi okupasi lubang tanam sesi ini.".to_string(),
                                impact_value: impact,
                                related_id: s.session_id.clone(),
                                target_path: format!("/finance/hpp/{}", s.session_id),
                                detected_at: scanned_at,
                            });
                        }
                    }
                }
            }
        }
    }

    insights
}

pub fn detect_sales_anomalies(
    sales: &[SaleAuditRow],
    sessions: &[HppSessionItem],
    today: NaiveDate,
    scanned_at: DateTime<Utc>,
) -> Vec<AiAuditInsight> {
    let mut insights = Vec::new();

    let session_cost_map: HashMap<&str, Decimal> = sessions
        .iter()
        .map(|s| (s.session_id.as_str(), s.cost_per_kg))
        .collect();

    for s in sales {
        // 3C. Overdue Accounts Receivable (payment_status == 'Piutang' and > 30 days)
        if s.payment_status.eq_ignore_ascii_case("Piutang") {
            let days_overdue = (today - s.sale_date).num_days();
            if days_overdue > 30 {
                insights.push(AiAuditInsight {
                    id: format!("sls_overdue_{}", s.sale_id),
                    category: AnomalyCategory::SALES,
                    severity: AnomalySeverity::Warning,
                    title: format!("Piutang Penjualan Jatuh Tempo ({} Hari)", days_overdue),
                    message: format!(
                        "Faktur penjualan {} senilai Rp {} berstatus Piutang telah berumur {} hari sejak tanggal transaksi.",
                        s.sale_number, s.net_proceeds, days_overdue
                    ),
                    suggestion: "Lakukan penagihan langsung ke pihak pelanggan atau sesuaikan batas kredit pembeli.".to_string(),
                    impact_value: s.net_proceeds,
                    related_id: s.sale_id.clone(),
                    target_path: "/finance/sales".to_string(),
                    detected_at: scanned_at,
                });
            }
        }

        // 3A & 3B. Margin & Loss check on Realized sales with unit KG
        if s.status.eq_ignore_ascii_case("Realized")
            && s.unit.trim().eq_ignore_ascii_case("KG")
            && s.quantity > Decimal::ZERO
        {
            if let Some(&cost_per_kg) = session_cost_map.get(s.session_id.as_str()) {
                if cost_per_kg > Decimal::ZERO {
                    let net_price_per_kg = round_to_two_decimals(s.net_proceeds / s.quantity);

                    if net_price_per_kg < cost_per_kg {
                        // 3A. Sold below cost (Critical)
                        let loss_diff = cost_per_kg - net_price_per_kg;
                        let impact = round_to_two_decimals(loss_diff * s.quantity);

                        insights.push(AiAuditInsight {
                            id: format!("sls_loss_{}", s.sale_id),
                            category: AnomalyCategory::SALES,
                            severity: AnomalySeverity::Critical,
                            title: "Penjualan Di Bawah Modal HPP".to_string(),
                            message: format!(
                                "Transaksi {} terjual bersih Rp {}/KG, lebih rendah dari HPP sesi (Rp {}/KG).",
                                s.sale_number, net_price_per_kg, cost_per_kg
                            ),
                            suggestion: "Evaluasi kebijakan diskon dan subsidi ongkir penjual pada kanal penjualan terkait atau naikkan harga jual.".to_string(),
                            impact_value: impact,
                            related_id: s.sale_id.clone(),
                            target_path: "/finance/sales".to_string(),
                            detected_at: scanned_at,
                        });
                    } else {
                        // 3B. Thin margin < 10% (Warning)
                        let margin_pct = if net_price_per_kg > Decimal::ZERO {
                            round_to_two_decimals(
                                ((net_price_per_kg - cost_per_kg) / net_price_per_kg)
                                    * Decimal::from(100),
                            )
                        } else {
                            Decimal::ZERO
                        };

                        if margin_pct < Decimal::from(10) {
                            // Target price to reach 10% margin: target_p = cost / 0.90
                            let target_p = round_to_two_decimals(
                                cost_per_kg / Decimal::from_str_exact("0.90").unwrap(),
                            );
                            let diff = if target_p > net_price_per_kg {
                                target_p - net_price_per_kg
                            } else {
                                Decimal::ZERO
                            };
                            let impact = round_to_two_decimals(diff * s.quantity);

                            insights.push(AiAuditInsight {
                                id: format!("sls_margin_{}", s.sale_id),
                                category: AnomalyCategory::SALES,
                                severity: AnomalySeverity::Warning,
                                title: format!("Margin Penjualan Tipis ({}%)", margin_pct),
                                message: format!(
                                    "Transaksi {} memperoleh margin {}%, di bawah target batas sehat 10%.",
                                    s.sale_number, margin_pct
                                ),
                                suggestion: "Pertimbangkan untuk mengurangi diskon kuantitas atau menaikkan harga dasar secara bertahap.".to_string(),
                                impact_value: impact,
                                related_id: s.sale_id.clone(),
                                target_path: "/finance/sales".to_string(),
                                detected_at: scanned_at,
                            });
                        }
                    }
                }
            }
        }
    }

    insights
}

pub fn detect_inventory_anomalies(
    items: &[InventoryAuditItemRow],
    scanned_at: DateTime<Utc>,
) -> Vec<AiAuditInsight> {
    let mut insights = Vec::new();

    for itm in items {
        // 4A. Stockout or Low stock
        if itm.min_stock_level > Decimal::ZERO && itm.current_stock <= itm.min_stock_level {
            if itm.current_stock == Decimal::ZERO {
                let impact = round_to_two_decimals(itm.min_stock_level * itm.average_unit_cost);
                insights.push(AiAuditInsight {
                    id: format!("inv_stockout_{}", itm.item_id),
                    category: AnomalyCategory::INVENTORY,
                    severity: AnomalySeverity::Critical,
                    title: format!("Stok Habis: {}", itm.item_name),
                    message: format!(
                        "Saldo stok barang {} saat ini 0 {}, batas minimum adalah {} {}.",
                        itm.item_name, itm.default_unit, itm.min_stock_level, itm.default_unit
                    ),
                    suggestion: "Segera lakukan pengadaan pembelian barang ini untuk menghindari gangguan jadwal operasional kebun.".to_string(),
                    impact_value: impact,
                    related_id: itm.item_id.clone(),
                    target_path: "/production/inventory".to_string(),
                    detected_at: scanned_at,
                });
            } else {
                let shortage = itm.min_stock_level - itm.current_stock;
                let impact = round_to_two_decimals(shortage * itm.average_unit_cost);
                insights.push(AiAuditInsight {
                    id: format!("inv_low_{}", itm.item_id),
                    category: AnomalyCategory::INVENTORY,
                    severity: AnomalySeverity::Warning,
                    title: format!("Stok Menipis Di Bawah Batas Minimum: {}", itm.item_name),
                    message: format!(
                        "Saldo stok barang {} tersisa {} {}, di bawah batas minimum {} {}.",
                        itm.item_name,
                        itm.current_stock,
                        itm.default_unit,
                        itm.min_stock_level,
                        itm.default_unit
                    ),
                    suggestion:
                        "Rencanakan pembuatan pesanan pembelian bahan ke pemasok dalam minggu ini."
                            .to_string(),
                    impact_value: impact,
                    related_id: itm.item_id.clone(),
                    target_path: "/production/inventory".to_string(),
                    detected_at: scanned_at,
                });
            }
        }

        // 4B. Dead / Slow-moving stock (> 60 days age and no STOCK_OUT in 60 days)
        if itm.current_stock > Decimal::ZERO {
            let item_age_days = (scanned_at - itm.item_created_at).num_days();
            if item_age_days > 60 {
                let no_stock_out_60_days = match itm.last_stock_out_at {
                    Some(last_out) => (scanned_at - last_out).num_days() > 60,
                    None => true,
                };

                if no_stock_out_60_days {
                    let impact = round_to_two_decimals(itm.current_stock * itm.average_unit_cost);
                    insights.push(AiAuditInsight {
                        id: format!("inv_dead_{}", itm.item_id),
                        category: AnomalyCategory::INVENTORY,
                        severity: AnomalySeverity::Info,
                        title: "Stok Gudang Tidak Bergerak (> 60 Hari)".to_string(),
                        message: format!(
                            "Item {} (saldo {} {}) tidak mencatat mutasi keluar selama lebih dari 60 hari.",
                            itm.item_name, itm.current_stock, itm.default_unit
                        ),
                        suggestion: "Audit kondisi fisik barang melalui Stock Opname dan prioritaskan pemakaian pada sesi pembibitan berikutnya.".to_string(),
                        impact_value: impact,
                        related_id: itm.item_id.clone(),
                        target_path: "/production/inventory".to_string(),
                        detected_at: scanned_at,
                    });
                }
            }
        }
    }

    insights
}

pub fn calculate_health_score_and_stats(
    insights: &[AiAuditInsight],
    scanned_at: DateTime<Utc>,
) -> AiAuditStats {
    let mut critical = 0i64;
    let mut warning = 0i64;
    let mut info = 0i64;
    let mut total_impact_rp = Decimal::ZERO;

    for ins in insights {
        match ins.severity {
            AnomalySeverity::Critical => critical += 1,
            AnomalySeverity::Warning => warning += 1,
            AnomalySeverity::Info => info += 1,
        }
        total_impact_rp += ins.impact_value;
    }

    let penalty = (15 * critical) + (5 * warning) + (2 * info);
    let health_score = (100 - penalty).clamp(0, 100);

    AiAuditStats {
        health_score,
        critical,
        warning,
        info,
        total_anomalies: insights.len() as i64,
        total_impact_rp: round_to_two_decimals(total_impact_rp),
        scanned_at,
    }
}

pub fn sort_insights(insights: &mut [AiAuditInsight]) {
    insights.sort_by(|a, b| {
        let rank_a = match a.severity {
            AnomalySeverity::Critical => 0,
            AnomalySeverity::Warning => 1,
            AnomalySeverity::Info => 2,
        };
        let rank_b = match b.severity {
            AnomalySeverity::Critical => 0,
            AnomalySeverity::Warning => 1,
            AnomalySeverity::Info => 2,
        };

        rank_a
            .cmp(&rank_b)
            .then_with(|| b.impact_value.cmp(&a.impact_value))
            .then_with(|| a.id.cmp(&b.id))
    });
}

#[async_trait]
pub trait AiAuditService: Send + Sync {
    async fn scan(&self, query: &AiAuditScanQuery) -> Result<AiAuditScanResponse, AppError>;
}

pub struct AiAuditServiceImpl {
    repo: Arc<dyn AiAuditRepository>,
    hpp_service: Arc<dyn HppService>,
}

impl AiAuditServiceImpl {
    pub fn new(repo: Arc<dyn AiAuditRepository>, hpp_service: Arc<dyn HppService>) -> Self {
        Self { repo, hpp_service }
    }
}

#[async_trait]
impl AiAuditService for AiAuditServiceImpl {
    async fn scan(&self, query: &AiAuditScanQuery) -> Result<AiAuditScanResponse, AppError> {
        let scanned_at = Utc::now();
        let today = today_wib();
        let cutoff_90_days = today - Duration::days(90);

        // Fetch all required data across pillars
        let purchases = self
            .repo
            .fetch_purchase_items_history(cutoff_90_days)
            .await?;
        let sales = self.repo.fetch_sales_history(cutoff_90_days).await?;
        let inventory_items = self.repo.fetch_inventory_audit_items().await?;

        // Reusing HppService to get up-to-date HPP session economics
        let hpp_res = self
            .hpp_service
            .get_hpp_list(&HppListQuery {
                location_id: None,
                status: None,
                search: None,
                date: Some(today),
                sort_order: None,
            })
            .await?;

        let mut all_insights = Vec::new();
        all_insights.extend(detect_purchase_price_spikes(&purchases, scanned_at));
        all_insights.extend(detect_hpp_anomalies(&hpp_res.items, scanned_at));
        all_insights.extend(detect_sales_anomalies(
            &sales,
            &hpp_res.items,
            today,
            scanned_at,
        ));
        all_insights.extend(detect_inventory_anomalies(&inventory_items, scanned_at));

        // Stats are computed across ALL anomalies in the entire system
        let stats = calculate_health_score_and_stats(&all_insights, scanned_at);

        // Sort insights by priority
        sort_insights(&mut all_insights);

        // Filter insights if query.category is provided
        let filtered_insights = if let Some(cat) = query.category {
            all_insights
                .into_iter()
                .filter(|ins| ins.category == cat)
                .collect()
        } else {
            all_insights
        };

        Ok(AiAuditScanResponse {
            stats,
            insights: filtered_insights,
        })
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
    fn test_health_score_clamping_and_weights() {
        let now = Utc::now();

        // 1 critical, 2 warning, 1 info -> 100 - (15*1 + 5*2 + 2*1) = 100 - 27 = 73
        let insights = vec![
            AiAuditInsight {
                id: "1".into(),
                category: AnomalyCategory::PURCHASE,
                severity: AnomalySeverity::Critical,
                title: "".into(),
                message: "".into(),
                suggestion: "".into(),
                impact_value: d("100.00"),
                related_id: "".into(),
                target_path: "".into(),
                detected_at: now,
            },
            AiAuditInsight {
                id: "2".into(),
                category: AnomalyCategory::HPP,
                severity: AnomalySeverity::Warning,
                title: "".into(),
                message: "".into(),
                suggestion: "".into(),
                impact_value: d("50.00"),
                related_id: "".into(),
                target_path: "".into(),
                detected_at: now,
            },
            AiAuditInsight {
                id: "3".into(),
                category: AnomalyCategory::SALES,
                severity: AnomalySeverity::Warning,
                title: "".into(),
                message: "".into(),
                suggestion: "".into(),
                impact_value: d("50.00"),
                related_id: "".into(),
                target_path: "".into(),
                detected_at: now,
            },
            AiAuditInsight {
                id: "4".into(),
                category: AnomalyCategory::INVENTORY,
                severity: AnomalySeverity::Info,
                title: "".into(),
                message: "".into(),
                suggestion: "".into(),
                impact_value: d("20.00"),
                related_id: "".into(),
                target_path: "".into(),
                detected_at: now,
            },
        ];

        let stats = calculate_health_score_and_stats(&insights, now);
        assert_eq!(stats.health_score, 73);
        assert_eq!(stats.critical, 1);
        assert_eq!(stats.warning, 2);
        assert_eq!(stats.info, 1);
        assert_eq!(stats.total_anomalies, 4);
        assert_eq!(stats.total_impact_rp, d("220.00"));

        // Extreme critical count clamps at 0
        let extreme = vec![insights[0].clone(); 10];
        let extreme_stats = calculate_health_score_and_stats(&extreme, now);
        assert_eq!(extreme_stats.health_score, 0);

        // 0 anomalies -> 100
        let empty_stats = calculate_health_score_and_stats(&[], now);
        assert_eq!(empty_stats.health_score, 100);
        assert_eq!(empty_stats.total_impact_rp, Decimal::ZERO);
    }

    #[test]
    fn test_rule_1_purchase_price_spike_detection() {
        let now = Utc::now();
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();

        let rows = vec![
            PurchaseItemHistoryRow {
                purchase_item_id: "pit_latest".into(),
                purchase_id: "prc_latest".into(),
                invoice_number: "PRC-02".into(),
                transaction_date: today,
                item_name: "Nutrisi AB Mix 5L".into(),
                unit: "PACK".into(),
                quantity: d("4.00"),
                total_cost: d("650000.00"),
                effective_unit_cost: d("162500.00"),
                created_at: now,
            },
            PurchaseItemHistoryRow {
                purchase_item_id: "pit_prev1".into(),
                purchase_id: "prc_prev1".into(),
                invoice_number: "PRC-01B".into(),
                transaction_date: today - Duration::days(10),
                item_name: "Nutrisi AB Mix 5L".into(),
                unit: "PACK".into(),
                quantity: d("2.00"),
                total_cost: d("260000.00"),
                effective_unit_cost: d("130000.00"),
                created_at: now - Duration::days(10),
            },
            PurchaseItemHistoryRow {
                purchase_item_id: "pit_prev2".into(),
                purchase_id: "prc_prev2".into(),
                invoice_number: "PRC-01A".into(),
                transaction_date: today - Duration::days(20),
                item_name: "Nutrisi AB Mix 5L".into(),
                unit: "PACK".into(),
                quantity: d("2.00"),
                total_cost: d("240000.00"),
                effective_unit_cost: d("120000.00"),
                created_at: now - Duration::days(20),
            },
        ];

        let insights = detect_purchase_price_spikes(&rows, now);
        assert_eq!(insights.len(), 1);
        let ins = &insights[0];
        // Hist avg = 125,000, latest = 162,500 (+30% -> warning)
        assert_eq!(ins.severity, AnomalySeverity::Warning);
        // impact = (162500 - 125000) * 4 = 150000.00
        assert_eq!(ins.impact_value, d("150000.00"));
    }

    #[test]
    fn test_rule_2a_hpp_spike_detection() {
        let now = Utc::now();
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();

        // 3 Melon sessions: Melon A (cost/kg 18000), Melon B (cost/kg 20000), Melon C (cost/kg 32000, yield 50 kg)
        // Group avg = (18000 + 20000 + 32000) / 3 = 23333.33
        // Melon C diff = 32000 - 23333.33 = 8666.67 (+37.14% > 25% -> critical)
        // Impact = round_2(8666.67 * 50) = 433333.50
        let sessions = vec![
            HppSessionItem {
                session_id: "ses_melon_a".into(),
                session_name: "Sesi Melon A".into(),
                plant_name: "Melon".into(),
                plant_variety: "Golden".into(),
                location_id: "loc_1".into(),
                location_name: "GH 1".into(),
                status: "Completed".into(),
                start_date: today - Duration::days(60),
                end_date: today - Duration::days(10),
                harvest_date: Some(today - Duration::days(10)),
                duration_days: 50,
                total_holes: 200,
                waste_qty: 10,
                waste_percentage: d("5.00"),
                direct_costs: d("900000.00"),
                allocated_shared_costs: Decimal::ZERO,
                allocated_asset_costs: Decimal::ZERO,
                allocated_costs: Decimal::ZERO,
                total_hpp: d("900000.00"),
                total_revenue: d("1500000.00"),
                total_shipping_subsidy: Decimal::ZERO,
                net_revenue: d("1500000.00"),
                net_profit: d("600000.00"),
                profit_margin_percent: d("40.00"),
                effective_yield_kg: d("50.00"),
                cost_per_kg: d("18000.00"),
                net_revenue_per_kg: d("30000.00"),
                profit_per_kg: d("12000.00"),
                is_snapshot_locked: true,
                hpp_snapshot_locked_at: Some(now),
            },
            HppSessionItem {
                session_id: "ses_melon_b".into(),
                session_name: "Sesi Melon B".into(),
                plant_name: "Melon".into(),
                plant_variety: "Golden".into(),
                location_id: "loc_1".into(),
                location_name: "GH 1".into(),
                status: "Completed".into(),
                start_date: today - Duration::days(60),
                end_date: today - Duration::days(10),
                harvest_date: Some(today - Duration::days(10)),
                duration_days: 50,
                total_holes: 200,
                waste_qty: 10,
                waste_percentage: d("5.00"),
                direct_costs: d("1000000.00"),
                allocated_shared_costs: Decimal::ZERO,
                allocated_asset_costs: Decimal::ZERO,
                allocated_costs: Decimal::ZERO,
                total_hpp: d("1000000.00"),
                total_revenue: d("1500000.00"),
                total_shipping_subsidy: Decimal::ZERO,
                net_revenue: d("1500000.00"),
                net_profit: d("500000.00"),
                profit_margin_percent: d("33.33"),
                effective_yield_kg: d("50.00"),
                cost_per_kg: d("20000.00"),
                net_revenue_per_kg: d("30000.00"),
                profit_per_kg: d("10000.00"),
                is_snapshot_locked: true,
                hpp_snapshot_locked_at: Some(now),
            },
            HppSessionItem {
                session_id: "ses_melon_c".into(),
                session_name: "Sesi Melon C".into(),
                plant_name: "Melon".into(),
                plant_variety: "Golden".into(),
                location_id: "loc_1".into(),
                location_name: "GH 1".into(),
                status: "Completed".into(),
                start_date: today - Duration::days(60),
                end_date: today - Duration::days(10),
                harvest_date: Some(today - Duration::days(10)),
                duration_days: 50,
                total_holes: 200,
                waste_qty: 10,
                waste_percentage: d("5.00"),
                direct_costs: d("1600000.00"),
                allocated_shared_costs: Decimal::ZERO,
                allocated_asset_costs: Decimal::ZERO,
                allocated_costs: Decimal::ZERO,
                total_hpp: d("1600000.00"),
                total_revenue: d("1800000.00"),
                total_shipping_subsidy: Decimal::ZERO,
                net_revenue: d("1800000.00"),
                net_profit: d("200000.00"),
                profit_margin_percent: d("11.11"),
                effective_yield_kg: d("50.00"),
                cost_per_kg: d("32000.00"),
                net_revenue_per_kg: d("36000.00"),
                profit_per_kg: d("4000.00"),
                is_snapshot_locked: true,
                hpp_snapshot_locked_at: Some(now),
            },
        ];

        let insights = detect_hpp_anomalies(&sessions, now);
        let spike_insights: Vec<&AiAuditInsight> = insights
            .iter()
            .filter(|i| i.id.starts_with("hpp_spike_"))
            .collect();

        assert_eq!(spike_insights.len(), 1);
        let c_ins = spike_insights[0];
        assert_eq!(c_ins.related_id, "ses_melon_c");
        assert_eq!(c_ins.severity, AnomalySeverity::Critical);
        assert_eq!(c_ins.impact_value, d("433333.50"));
    }

    #[test]
    fn test_rule_3a_and_3b_sales_margin_and_unit_pack_skip() {
        let now = Utc::now();
        let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();

        let session = HppSessionItem {
            session_id: "ses_s1".into(),
            session_name: "Sesi 1".into(),
            plant_name: "Selada".into(),
            plant_variety: "Romaine".into(),
            location_id: "loc_1".into(),
            location_name: "GH 1".into(),
            status: "Completed".into(),
            start_date: today - Duration::days(40),
            end_date: today - Duration::days(5),
            harvest_date: Some(today - Duration::days(5)),
            duration_days: 35,
            total_holes: 500,
            waste_qty: 25,
            waste_percentage: d("5.00"),
            direct_costs: d("2000000.00"),
            allocated_shared_costs: Decimal::ZERO,
            allocated_asset_costs: Decimal::ZERO,
            allocated_costs: Decimal::ZERO,
            total_hpp: d("2000000.00"),
            total_revenue: d("3000000.00"),
            total_shipping_subsidy: Decimal::ZERO,
            net_revenue: d("3000000.00"),
            net_profit: d("1000000.00"),
            profit_margin_percent: d("33.33"),
            effective_yield_kg: d("100.00"),
            cost_per_kg: d("20000.00"),
            net_revenue_per_kg: d("30000.00"),
            profit_per_kg: d("10000.00"),
            is_snapshot_locked: true,
            hpp_snapshot_locked_at: Some(now),
        };

        // Penjualan A: qty 20 KG, rev 420.000, subsidy 40.000 -> net 380.000 -> P_net = 19.000 < 20.000 (Loss)
        // Impact = (20000 - 19000) * 20 = 20000.00
        let sale_a = SaleAuditRow {
            sale_id: "sls_a".into(),
            sale_number: "SLS-A".into(),
            session_id: "ses_s1".into(),
            sale_date: today,
            unit: "KG".into(),
            quantity: d("20.00"),
            total_revenue: d("420000.00"),
            seller_shipping_subsidy: d("40000.00"),
            net_proceeds: d("380000.00"),
            status: "Realized".into(),
            payment_status: "Lunas".into(),
        };

        // Penjualan B: qty 10 KG, net 210.000 -> P_net = 21.000 >= 20.000, margin = (21000-20000)/21000*100 = 4.76% < 10%
        // Target P = 20000 / 0.90 = 22222.22, diff = 1222.22, Impact = 1222.22 * 10 = 12222.20
        let sale_b = SaleAuditRow {
            sale_id: "sls_b".into(),
            sale_number: "SLS-B".into(),
            session_id: "ses_s1".into(),
            sale_date: today,
            unit: "KG".into(),
            quantity: d("10.00"),
            total_revenue: d("210000.00"),
            seller_shipping_subsidy: Decimal::ZERO,
            net_proceeds: d("210000.00"),
            status: "Realized".into(),
            payment_status: "Lunas".into(),
        };

        // Penjualan C: unit "PACK" -> wajib di-skip dari perbandingan cost_per_kg
        let sale_c = SaleAuditRow {
            sale_id: "sls_c".into(),
            sale_number: "SLS-C".into(),
            session_id: "ses_s1".into(),
            sale_date: today,
            unit: "PACK".into(),
            quantity: d("50.00"),
            total_revenue: d("500000.00"),
            seller_shipping_subsidy: Decimal::ZERO,
            net_proceeds: d("500000.00"),
            status: "Realized".into(),
            payment_status: "Lunas".into(),
        };

        let sales = vec![sale_a, sale_b, sale_c];
        let sessions = vec![session];

        let insights = detect_sales_anomalies(&sales, &sessions, today, now);

        // Expect exactly 2 insights: 1 loss (Critical) and 1 thin margin (Warning), sale_c skipped
        assert_eq!(insights.len(), 2);

        let loss_ins = insights.iter().find(|i| i.id == "sls_loss_sls_a").unwrap();
        assert_eq!(loss_ins.severity, AnomalySeverity::Critical);
        assert_eq!(loss_ins.impact_value, d("20000.00"));

        let margin_ins = insights
            .iter()
            .find(|i| i.id == "sls_margin_sls_b")
            .unwrap();
        assert_eq!(margin_ins.severity, AnomalySeverity::Warning);
        assert_eq!(margin_ins.impact_value, d("12222.20"));
    }

    #[test]
    fn test_rule_4a_inventory_stockout_and_low_stock() {
        let now = Utc::now();

        // Item X: current_stock = 0, min_stock_level = 10, average_unit_cost = 15000
        // Stockout (Critical), Impact = 10 * 15000 = 150000.00
        let item_x = InventoryAuditItemRow {
            item_id: "itm_x".into(),
            item_name: "Rockwool".into(),
            min_stock_level: d("10.00"),
            current_stock: Decimal::ZERO,
            average_unit_cost: d("15000.00"),
            default_unit: "SLAB".into(),
            item_created_at: now - Duration::days(100),
            last_stock_out_at: Some(now - Duration::days(10)),
        };

        // Item Y: current_stock = 4, min_stock_level = 10, average_unit_cost = 15000
        // Low stock (Warning), Impact = (10 - 4) * 15000 = 90000.00
        let item_y = InventoryAuditItemRow {
            item_id: "itm_y".into(),
            item_name: "Netpot 5cm".into(),
            min_stock_level: d("10.00"),
            current_stock: d("4.00"),
            average_unit_cost: d("15000.00"),
            default_unit: "PCS".into(),
            item_created_at: now - Duration::days(100),
            last_stock_out_at: Some(now - Duration::days(10)),
        };

        let items = vec![item_x, item_y];
        let insights = detect_inventory_anomalies(&items, now);

        assert_eq!(insights.len(), 2);

        let stockout_ins = insights
            .iter()
            .find(|i| i.id == "inv_stockout_itm_x")
            .unwrap();
        assert_eq!(stockout_ins.severity, AnomalySeverity::Critical);
        assert_eq!(stockout_ins.impact_value, d("150000.00"));

        let low_ins = insights.iter().find(|i| i.id == "inv_low_itm_y").unwrap();
        assert_eq!(low_ins.severity, AnomalySeverity::Warning);
        assert_eq!(low_ins.impact_value, d("90000.00"));
    }
}
