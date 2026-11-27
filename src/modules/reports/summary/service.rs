use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;
use crate::modules::reports::summary::dto::{
    CashflowEntryType, CashflowLedgerEntry, CashflowPeriodMeta, CashflowSummaryKpi,
    CashflowSummaryQuery, CashflowSummaryResponse, CashflowTypeFilter, SortOrder,
};
use crate::modules::reports::summary::repository::{
    PurchaseCashflowRow, ReportSummaryRepository, SaleCashflowRow,
};

pub fn build_cashflow_kpi(
    sales: &[SaleCashflowRow],
    purchases: &[PurchaseCashflowRow],
) -> CashflowSummaryKpi {
    let mut total_gross_sales = Decimal::ZERO;
    let mut total_shipping_subsidy = Decimal::ZERO;
    let mut total_inflow = Decimal::ZERO;

    for s in sales {
        total_gross_sales += s.total_revenue;
        total_shipping_subsidy += s.seller_shipping_subsidy;
        total_inflow += s.net_proceeds;
    }

    let mut total_outflow = Decimal::ZERO;
    for p in purchases {
        total_outflow += p.total_amount;
    }

    total_gross_sales = round_to_two_decimals(total_gross_sales);
    total_shipping_subsidy = round_to_two_decimals(total_shipping_subsidy);
    total_inflow = round_to_two_decimals(total_inflow);
    total_outflow = round_to_two_decimals(total_outflow);

    let net_cashflow = round_to_two_decimals(total_inflow - total_outflow);
    let profit_margin_percent = if total_inflow > Decimal::ZERO {
        round_to_two_decimals((net_cashflow / total_inflow) * Decimal::from(100))
    } else {
        Decimal::ZERO
    };

    CashflowSummaryKpi {
        total_gross_sales,
        total_shipping_subsidy,
        total_inflow,
        total_outflow,
        net_cashflow,
        profit_margin_percent,
        inflow_count: sales.len() as i64,
        outflow_count: purchases.len() as i64,
        total_entries: (sales.len() + purchases.len()) as i64,
    }
}

pub fn convert_sale_to_entry(s: &SaleCashflowRow) -> CashflowLedgerEntry {
    let description = format!(
        "Penjualan Sesi {} ({} {})",
        s.session_name, s.quantity, s.unit
    );

    CashflowLedgerEntry {
        id: s.id.clone(),
        date: s.sale_date,
        entry_type: CashflowEntryType::INFLOW,
        category: "Penjualan".to_string(),
        reference_number: s.sale_number.clone(),
        counterparty: s.counterparty.clone(),
        location_id: s.location_id.clone(),
        location_name: s.location_name.clone(),
        description,
        gross_amount: s.total_revenue,
        shipping_subsidy: s.seller_shipping_subsidy,
        amount: s.net_proceeds,
        status: Some(s.status.clone()),
        payment_status: Some(s.payment_status.clone()),
        target_path: "/finance/sales".to_string(),
    }
}

pub fn convert_purchase_to_entry(p: &PurchaseCashflowRow) -> CashflowLedgerEntry {
    let description = if let Some(first_item) = &p.first_item_name {
        if p.item_count > 1 {
            format!(
                "Pembelian {} (+{} item lainnya)",
                first_item,
                p.item_count - 1
            )
        } else {
            format!("Pembelian {}", first_item)
        }
    } else if let Some(notes) = &p.notes {
        if !notes.trim().is_empty() {
            notes.clone()
        } else {
            "Pembelian Bahan / Sarana".to_string()
        }
    } else {
        "Pembelian Bahan / Sarana".to_string()
    };

    CashflowLedgerEntry {
        id: p.id.clone(),
        date: p.transaction_date,
        entry_type: CashflowEntryType::OUTFLOW,
        category: "Pembelian".to_string(),
        reference_number: p.invoice_number.clone(),
        counterparty: p.counterparty.clone(),
        location_id: p.location_id.clone(),
        location_name: p.location_name.clone(),
        description,
        gross_amount: p.total_amount,
        shipping_subsidy: Decimal::ZERO,
        amount: p.total_amount,
        status: None,
        payment_status: None,
        target_path: "/finance/purchases".to_string(),
    }
}

#[async_trait]
pub trait ReportSummaryService: Send + Sync {
    async fn get_cashflow_summary(
        &self,
        query: &CashflowSummaryQuery,
    ) -> Result<CashflowSummaryResponse, AppError>;
}

pub struct ReportSummaryServiceImpl {
    repo: Arc<dyn ReportSummaryRepository>,
}

impl ReportSummaryServiceImpl {
    pub fn new(repo: Arc<dyn ReportSummaryRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ReportSummaryService for ReportSummaryServiceImpl {
    async fn get_cashflow_summary(
        &self,
        query: &CashflowSummaryQuery,
    ) -> Result<CashflowSummaryResponse, AppError> {
        let today = today_wib();
        let start_date = query
            .start_date
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap());
        let end_date = query.end_date.unwrap_or(today);

        if start_date > end_date {
            return Err(AppError::BadRequest(
                "Tanggal mulai tidak boleh lebih besar dari tanggal akhir".into(),
            ));
        }

        let loc_id = query
            .location_id
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let realized_only = query.realized_only.unwrap_or(true);
        let type_filter = query.type_filter.unwrap_or(CashflowTypeFilter::ALL);
        let sort_order = query.sort_order.unwrap_or(SortOrder::DESC);

        let sales = self
            .repo
            .fetch_sales_inflows(start_date, end_date, loc_id, realized_only)
            .await?;
        let purchases = self
            .repo
            .fetch_purchase_outflows(start_date, end_date, loc_id)
            .await?;

        // Optional search filtering on raw rows before KPI & entries
        let search_term = query
            .search
            .as_deref()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty());

        let filtered_sales: Vec<SaleCashflowRow> = if let Some(term) = &search_term {
            sales
                .into_iter()
                .filter(|s| {
                    s.sale_number.to_lowercase().contains(term)
                        || s.counterparty.to_lowercase().contains(term)
                        || s.session_name.to_lowercase().contains(term)
                        || s.location_name.to_lowercase().contains(term)
                })
                .collect()
        } else {
            sales
        };

        let filtered_purchases: Vec<PurchaseCashflowRow> = if let Some(term) = &search_term {
            purchases
                .into_iter()
                .filter(|p| {
                    p.invoice_number.to_lowercase().contains(term)
                        || p.counterparty.to_lowercase().contains(term)
                        || p.first_item_name
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(term)
                        || p.notes
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(term)
                        || p.location_name.to_lowercase().contains(term)
                })
                .collect()
        } else {
            purchases
        };

        // KPI is calculated across ALL matching transactions in the period
        let kpi = build_cashflow_kpi(&filtered_sales, &filtered_purchases);

        // Entries are mapped and filtered by type_filter
        let mut entries: Vec<CashflowLedgerEntry> = Vec::new();

        if type_filter == CashflowTypeFilter::ALL || type_filter == CashflowTypeFilter::INFLOW {
            for s in &filtered_sales {
                entries.push(convert_sale_to_entry(s));
            }
        }

        if type_filter == CashflowTypeFilter::ALL || type_filter == CashflowTypeFilter::OUTFLOW {
            for p in &filtered_purchases {
                entries.push(convert_purchase_to_entry(p));
            }
        }

        // Sort entries by date, then id
        match sort_order {
            SortOrder::ASC => {
                entries.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.id.cmp(&b.id)));
            }
            SortOrder::DESC => {
                entries.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.id.cmp(&a.id)));
            }
        }

        Ok(CashflowSummaryResponse {
            period: CashflowPeriodMeta {
                start_date,
                end_date,
                location_id: loc_id.map(|s| s.to_string()),
                realized_only,
            },
            kpi,
            entries,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chrono::Utc;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_cashflow_summary_kpi_and_profit_margin() {
        let sale = SaleCashflowRow {
            id: "s1".into(),
            sale_date: NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(),
            sale_number: "SLS-01".into(),
            location_id: "loc1".into(),
            location_name: "GH 1".into(),
            session_name: "Sesi 1".into(),
            counterparty: "Cust 1".into(),
            quantity: d("100.00"),
            unit: "KG".into(),
            total_revenue: d("3000000.00"),
            seller_shipping_subsidy: d("100000.00"),
            net_proceeds: d("2900000.00"),
            status: "Realized".into(),
            payment_status: "Lunas".into(),
            created_at: Utc::now(),
        };

        let purchase = PurchaseCashflowRow {
            id: "p1".into(),
            transaction_date: NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
            invoice_number: "PRC-01".into(),
            location_id: "loc1".into(),
            location_name: "GH 1".into(),
            counterparty: "Sup 1".into(),
            total_amount: d("1160000.00"),
            first_item_name: Some("Nutrisi".into()),
            item_count: 1,
            notes: None,
            created_at: Utc::now(),
        };

        let kpi = build_cashflow_kpi(&[sale], &[purchase]);

        assert_eq!(kpi.total_gross_sales, d("3000000.00"));
        assert_eq!(kpi.total_shipping_subsidy, d("100000.00"));
        assert_eq!(kpi.total_inflow, d("2900000.00"));
        assert_eq!(kpi.total_outflow, d("1160000.00"));
        assert_eq!(kpi.net_cashflow, d("1740000.00"));
        // 1740000 / 2900000 * 100 = 60.00%
        assert_eq!(kpi.profit_margin_percent, d("60.00"));
        assert_eq!(kpi.inflow_count, 1);
        assert_eq!(kpi.outflow_count, 1);
        assert_eq!(kpi.total_entries, 2);
    }

    #[test]
    fn test_cashflow_zero_inflow_safe_division() {
        let purchase = PurchaseCashflowRow {
            id: "p1".into(),
            transaction_date: NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
            invoice_number: "PRC-01".into(),
            location_id: "loc1".into(),
            location_name: "GH 1".into(),
            counterparty: "Sup 1".into(),
            total_amount: d("500000.00"),
            first_item_name: None,
            item_count: 0,
            notes: None,
            created_at: Utc::now(),
        };

        let kpi = build_cashflow_kpi(&[], &[purchase]);
        assert_eq!(kpi.total_inflow, Decimal::ZERO);
        assert_eq!(kpi.total_outflow, d("500000.00"));
        assert_eq!(kpi.net_cashflow, d("-500000.00"));
        assert_eq!(kpi.profit_margin_percent, Decimal::ZERO);
    }
}
