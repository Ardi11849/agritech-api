use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;
use crate::modules::finance::purchases::dto::{
    AllocationType, CreatePurchaseRequest, PurchaseDistributionResult, PurchaseItemCalcInput,
    PurchaseItemDistribution, PurchaseListResponse, PurchaseQuery, PurchaseResponse,
};
use crate::modules::finance::purchases::repository::PurchaseRepository;

pub fn default_allocation_days(
    allocation_type: AllocationType,
    input: Option<i32>,
) -> Result<i32, AppError> {
    let days = match allocation_type {
        AllocationType::Direct => 1,
        AllocationType::Shared => input.unwrap_or(365),
        AllocationType::Asset => input.unwrap_or(730),
    };

    if !(1..=3650).contains(&days) {
        return Err(AppError::BadRequest(
            "Durasi alokasi biaya harus berada di antara 1 sampai 3650 hari".into(),
        ));
    }

    Ok(days)
}

pub fn format_document_number(prefix: &str, date: NaiveDate, seq: i32) -> String {
    format!("{prefix}-{}-{:03}", date.format("%Y%m%d"), seq)
}

pub fn calculate_purchase_distribution(
    items: &[PurchaseItemCalcInput],
    global_discount: Decimal,
    global_shipping: Decimal,
) -> Result<PurchaseDistributionResult, AppError> {
    if items.is_empty() {
        return Err(AppError::BadRequest(
            "Nota pembelian wajib memiliki minimal 1 item".into(),
        ));
    }
    if items.len() > 100 {
        return Err(AppError::BadRequest(
            "Nota pembelian maksimal memiliki 100 item".into(),
        ));
    }

    let max_amount = Decimal::from_str("999999999999.99")
        .map_err(|e| AppError::Internal(format!("Invalid decimal limit: {e}")))?;

    let g_discount = round_to_two_decimals(global_discount);
    let g_shipping = round_to_two_decimals(global_shipping);

    if g_discount < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Diskon global tidak boleh bernilai negatif".into(),
        ));
    }
    if g_shipping < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Ongkos kirim global tidak boleh bernilai negatif".into(),
        ));
    }
    if g_discount > max_amount || g_shipping > max_amount {
        return Err(AppError::BadRequest(
            "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
        ));
    }

    let mut subtotals = Vec::with_capacity(items.len());
    let mut subtotal_header = Decimal::ZERO;

    for (idx, item) in items.iter().enumerate() {
        let line_no = (idx + 1) as i32;
        let qty = round_to_two_decimals(item.quantity);
        let price = round_to_two_decimals(item.price_per_unit);

        if qty <= Decimal::ZERO {
            return Err(AppError::BadRequest(format!(
                "Kuantitas item pada baris #{line_no} harus lebih besar dari 0.00"
            )));
        }
        if price < Decimal::ZERO {
            return Err(AppError::BadRequest(format!(
                "Harga satuan item pada baris #{line_no} tidak boleh bernilai negatif"
            )));
        }

        let sub = round_to_two_decimals(qty * price);
        if sub > max_amount {
            return Err(AppError::BadRequest(
                "Subtotal item melebihi batas kapasitas sistem".into(),
            ));
        }

        subtotals.push(sub);
        subtotal_header += sub;
    }

    subtotal_header = round_to_two_decimals(subtotal_header);
    if subtotal_header > max_amount {
        return Err(AppError::BadRequest(
            "Subtotal nota pembelian melebihi batas kapasitas sistem".into(),
        ));
    }

    if g_discount > subtotal_header {
        return Err(AppError::BadRequest(
            "Diskon global tidak boleh melebihi subtotal nota pembelian".into(),
        ));
    }

    let total_amount = round_to_two_decimals(subtotal_header - g_discount + g_shipping);
    if total_amount > max_amount {
        return Err(AppError::BadRequest(
            "Total nota pembelian melebihi batas kapasitas sistem".into(),
        ));
    }

    let m = items.len();
    let mut distributed_items = Vec::with_capacity(m);
    let mut cum_discount = Decimal::ZERO;
    let mut cum_shipping = Decimal::ZERO;

    for i in 0..m {
        let line_number = (i + 1) as i32;
        let sub_i = subtotals[i];
        let item_input = &items[i];
        let qty_i = round_to_two_decimals(item_input.quantity);
        let alloc_days =
            default_allocation_days(item_input.allocation_type, item_input.allocation_days)?;

        let (disc_i, ship_i) = if i == m - 1 {
            // Penny reconciliation on the last item
            let d_m = round_to_two_decimals(g_discount - cum_discount);
            let s_m = round_to_two_decimals(g_shipping - cum_shipping);
            (d_m, s_m)
        } else {
            let ratio_i = if subtotal_header > Decimal::ZERO {
                sub_i / subtotal_header
            } else {
                Decimal::ONE / Decimal::from(m)
            };

            let d_i = round_to_two_decimals(g_discount * ratio_i);
            let s_i = round_to_two_decimals(g_shipping * ratio_i);
            cum_discount += d_i;
            cum_shipping += s_i;
            (d_i, s_i)
        };

        let total_cost_i = round_to_two_decimals(sub_i - disc_i + ship_i);
        let effective_unit_cost_i = round_to_two_decimals(total_cost_i / qty_i);

        let daily_rate_i = match item_input.allocation_type {
            AllocationType::Direct => total_cost_i,
            AllocationType::Shared | AllocationType::Asset => {
                round_to_two_decimals(total_cost_i / Decimal::from(alloc_days))
            }
        };

        distributed_items.push(PurchaseItemDistribution {
            line_number,
            subtotal: sub_i,
            discount: disc_i,
            shipping_cost: ship_i,
            total_cost: total_cost_i,
            effective_unit_cost: effective_unit_cost_i,
            allocation_days: alloc_days,
            daily_allocation_rate: daily_rate_i,
        });
    }

    Ok(PurchaseDistributionResult {
        subtotal_amount: subtotal_header,
        global_discount: g_discount,
        global_shipping: g_shipping,
        total_amount,
        items: distributed_items,
    })
}

#[async_trait]
pub trait PurchaseService: Send + Sync {
    async fn get_purchases(&self, query: &PurchaseQuery) -> Result<PurchaseListResponse, AppError>;
    async fn get_purchase_by_id(&self, id: &str) -> Result<PurchaseResponse, AppError>;
    async fn create_purchase(
        &self,
        req: CreatePurchaseRequest,
        actor: &str,
    ) -> Result<PurchaseResponse, AppError>;
    async fn delete_purchase(&self, id: &str, actor: &str) -> Result<(), AppError>;
}

pub struct PurchaseServiceImpl {
    repo: Arc<dyn PurchaseRepository>,
}

impl PurchaseServiceImpl {
    pub fn new(repo: Arc<dyn PurchaseRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl PurchaseService for PurchaseServiceImpl {
    async fn get_purchases(&self, query: &PurchaseQuery) -> Result<PurchaseListResponse, AppError> {
        self.repo.find_all(query).await
    }

    async fn get_purchase_by_id(&self, id: &str) -> Result<PurchaseResponse, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Purchase with ID '{id}' not found")))
    }

    async fn create_purchase(
        &self,
        req: CreatePurchaseRequest,
        actor: &str,
    ) -> Result<PurchaseResponse, AppError> {
        if req.location_id.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Lokasi tujuan pengadaan wajib diisi".into(),
            ));
        }

        let calc_inputs: Vec<PurchaseItemCalcInput> = req
            .items
            .iter()
            .map(|item| PurchaseItemCalcInput {
                quantity: item.quantity,
                price_per_unit: item.price_per_unit,
                allocation_type: item.allocation_type,
                allocation_days: item.allocation_days,
            })
            .collect();

        let dist_result = calculate_purchase_distribution(
            &calc_inputs,
            req.global_discount.unwrap_or(Decimal::ZERO),
            req.global_shipping.unwrap_or(Decimal::ZERO),
        )?;

        let tx_date = req.transaction_date.unwrap_or_else(today_wib);

        self.repo.create(req, dist_result, tx_date, actor).await
    }

    async fn delete_purchase(&self, id: &str, actor: &str) -> Result<(), AppError> {
        self.repo.delete(id, actor).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_purchase_simulasi_1_proportional_and_penny_reconciliation() {
        // Input Nota Pembelian Simulasi 1 PRD:
        // global_discount = 30000.00, global_shipping = 60000.00
        // Item 1: Q1 = 10.00 LTR, P1 = 20000.00, Direct
        // Item 2: Q2 = 5.00 PACK, P2 = 80000.00, Shared, 365 days
        let items = vec![
            PurchaseItemCalcInput {
                quantity: d("10.00"),
                price_per_unit: d("20000.00"),
                allocation_type: AllocationType::Direct,
                allocation_days: None,
            },
            PurchaseItemCalcInput {
                quantity: d("5.00"),
                price_per_unit: d("80000.00"),
                allocation_type: AllocationType::Shared,
                allocation_days: Some(365),
            },
        ];

        let result = calculate_purchase_distribution(&items, d("30000.00"), d("60000.00"))
            .expect("Calculation must succeed");

        // Assertions Header
        assert_eq!(result.subtotal_amount, d("600000.00"));
        assert_eq!(result.global_discount, d("30000.00"));
        assert_eq!(result.global_shipping, d("60000.00"));
        assert_eq!(result.total_amount, d("630000.00"));
        assert_eq!(result.items.len(), 2);

        // Assertions Item 1
        let it1 = &result.items[0];
        assert_eq!(it1.line_number, 1);
        assert_eq!(it1.subtotal, d("200000.00"));
        assert_eq!(it1.discount, d("10000.00"));
        assert_eq!(it1.shipping_cost, d("20000.00"));
        assert_eq!(it1.total_cost, d("210000.00"));
        assert_eq!(it1.effective_unit_cost, d("21000.00"));
        assert_eq!(it1.daily_allocation_rate, d("210000.00"));
        assert_eq!(it1.allocation_days, 1);

        // Assertions Item 2 (Penny Reconciliation on i = m = 2)
        let it2 = &result.items[1];
        assert_eq!(it2.line_number, 2);
        assert_eq!(it2.subtotal, d("400000.00"));
        assert_eq!(it2.discount, d("20000.00"));
        assert_eq!(it2.shipping_cost, d("40000.00"));
        assert_eq!(it2.total_cost, d("420000.00"));
        assert_eq!(it2.effective_unit_cost, d("84000.00"));
        assert_eq!(it2.daily_allocation_rate, d("1150.68"));
        assert_eq!(it2.allocation_days, 365);

        // Invariant Check: Sum(total_cost) == total_amount
        assert_eq!(it1.total_cost + it2.total_cost, result.total_amount);
    }

    #[test]
    fn test_discount_exceeding_subtotal_rejected() {
        let items = vec![PurchaseItemCalcInput {
            quantity: d("2.00"),
            price_per_unit: d("50000.00"),
            allocation_type: AllocationType::Direct,
            allocation_days: None,
        }];
        // Subtotal = 100000, discount = 100001 -> must error
        let err = calculate_purchase_distribution(&items, d("100001.00"), d("0.00"));
        assert!(err.is_err());
    }
}
