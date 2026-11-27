use async_trait::async_trait;
use rust_decimal::Decimal;
use std::cmp::min;
use std::str::FromStr;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::common::time::today_wib;
use crate::modules::finance::sales::dto::{
    CreateSaleRequest, SaleCalcInput, SaleFinancialResult, SaleListResponse, SaleQuery,
    SaleResponse, SaleStatus, ShippingBorneBy, UpdateSaleRequest,
};
use crate::modules::finance::sales::repository::SaleRepository;

pub fn calculate_sale_financials(input: &SaleCalcInput) -> Result<SaleFinancialResult, AppError> {
    let max_amount = Decimal::from_str("999999999999.99")
        .map_err(|e| AppError::Internal(format!("Invalid decimal limit: {e}")))?;
    let max_qty = Decimal::from_str("9999999999.99")
        .map_err(|e| AppError::Internal(format!("Invalid decimal limit: {e}")))?;

    let qty = round_to_two_decimals(input.quantity);
    let mut req_qty = round_to_two_decimals(input.requested_quantity.unwrap_or(Decimal::ZERO));

    if qty < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Kuantitas penjualan tidak boleh bernilai negatif".into(),
        ));
    }
    if req_qty < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Kuantitas permintaan tidak boleh bernilai negatif".into(),
        ));
    }
    if qty > max_qty || req_qty > max_qty {
        return Err(AppError::BadRequest(
            "Kuantitas penjualan melebihi batas kapasitas sistem".into(),
        ));
    }

    let (effective_qty, fulfillment_rate_percent) = match input.status {
        SaleStatus::Order => {
            if req_qty <= Decimal::ZERO {
                if qty > Decimal::ZERO {
                    req_qty = qty;
                } else {
                    return Err(AppError::BadRequest(
                        "Saat status Order, kuantitas permintaan (requested_quantity) harus lebih besar dari 0.00".into(),
                    ));
                }
            }

            let eff_qty = if qty > Decimal::ZERO { qty } else { req_qty };

            let fulfillment = if qty <= Decimal::ZERO {
                Decimal::ZERO
            } else {
                let hundred = Decimal::from(100);
                let raw_fulfillment = round_to_two_decimals((qty / req_qty) * hundred);
                min(hundred, raw_fulfillment)
            };

            (eff_qty, fulfillment)
        }
        SaleStatus::Realized => {
            if qty <= Decimal::ZERO {
                return Err(AppError::BadRequest(
                    "Saat status Realized, kuantitas penjualan (quantity) harus lebih besar dari 0.00".into(),
                ));
            }

            if req_qty <= Decimal::ZERO {
                req_qty = qty;
            }

            let hundred = Decimal::from(100);
            let raw_fulfillment = round_to_two_decimals((qty / req_qty) * hundred);
            let fulfillment = min(hundred, raw_fulfillment);

            (qty, fulfillment)
        }
    };

    let price = round_to_two_decimals(input.price_per_unit);
    if price < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Harga satuan tidak boleh bernilai negatif".into(),
        ));
    }
    if price > max_amount {
        return Err(AppError::BadRequest(
            "Harga satuan melebihi batas kapasitas sistem".into(),
        ));
    }

    let gross_revenue = round_to_two_decimals(effective_qty * price);
    if gross_revenue > max_amount {
        return Err(AppError::BadRequest(
            "Omzet kotor penjualan melebihi batas kapasitas sistem".into(),
        ));
    }

    let discount = round_to_two_decimals(input.discount.unwrap_or(Decimal::ZERO));
    if discount < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Diskon tidak boleh bernilai negatif".into(),
        ));
    }
    if discount > gross_revenue {
        return Err(AppError::BadRequest(
            "Diskon tidak boleh melebihi omzet kotor penjualan".into(),
        ));
    }

    let total_revenue = round_to_two_decimals(gross_revenue - discount);

    let shipping_cost = round_to_two_decimals(input.shipping_cost.unwrap_or(Decimal::ZERO));
    if shipping_cost < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Ongkos kirim tidak boleh bernilai negatif".into(),
        ));
    }
    if shipping_cost > max_amount {
        return Err(AppError::BadRequest(
            "Ongkos kirim melebihi batas kapasitas sistem".into(),
        ));
    }

    let seller_shipping_subsidy = match input.shipping_borne_by {
        ShippingBorneBy::Penjual => shipping_cost,
        ShippingBorneBy::Pembeli => Decimal::ZERO,
    };

    let net_proceeds = round_to_two_decimals(total_revenue - seller_shipping_subsidy);

    Ok(SaleFinancialResult {
        gross_revenue,
        discount,
        total_revenue,
        shipping_cost,
        seller_shipping_subsidy,
        net_proceeds,
        fulfillment_rate_percent,
        requested_quantity: req_qty,
        quantity: qty,
    })
}

#[async_trait]
pub trait SaleService: Send + Sync {
    async fn get_sales(&self, query: &SaleQuery) -> Result<SaleListResponse, AppError>;
    async fn get_sale_by_id(&self, id: &str) -> Result<SaleResponse, AppError>;
    async fn create_sale(
        &self,
        req: CreateSaleRequest,
        actor: &str,
    ) -> Result<SaleResponse, AppError>;
    async fn update_sale(
        &self,
        id: &str,
        req: UpdateSaleRequest,
        actor: &str,
    ) -> Result<SaleResponse, AppError>;
    async fn delete_sale(&self, id: &str) -> Result<(), AppError>;
}

pub struct SaleServiceImpl {
    repo: Arc<dyn SaleRepository>,
}

impl SaleServiceImpl {
    pub fn new(repo: Arc<dyn SaleRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SaleService for SaleServiceImpl {
    async fn get_sales(&self, query: &SaleQuery) -> Result<SaleListResponse, AppError> {
        self.repo.find_all(query).await
    }

    async fn get_sale_by_id(&self, id: &str) -> Result<SaleResponse, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Sale with ID '{id}' not found")))
    }

    async fn create_sale(
        &self,
        req: CreateSaleRequest,
        actor: &str,
    ) -> Result<SaleResponse, AppError> {
        if req.session_id.trim().is_empty() {
            return Err(AppError::BadRequest("Sesi tanam wajib dipilih".into()));
        }

        let status = req.status.unwrap_or(SaleStatus::Order);
        let shipping_borne = req.shipping_borne_by.unwrap_or(ShippingBorneBy::Pembeli);
        let calc_input = SaleCalcInput {
            quantity: req.quantity.unwrap_or(Decimal::ZERO),
            requested_quantity: req.requested_quantity,
            price_per_unit: req.price_per_unit,
            discount: req.discount,
            shipping_cost: req.shipping_cost,
            shipping_borne_by: shipping_borne,
            status,
        };

        let financials = calculate_sale_financials(&calc_input)?;
        let sale_date = req.sale_date.unwrap_or_else(today_wib);

        self.repo.create(req, financials, sale_date, actor).await
    }

    async fn update_sale(
        &self,
        id: &str,
        req: UpdateSaleRequest,
        actor: &str,
    ) -> Result<SaleResponse, AppError> {
        if req.session_id.trim().is_empty() {
            return Err(AppError::BadRequest("Sesi tanam wajib dipilih".into()));
        }

        let status = req.status.unwrap_or(SaleStatus::Order);
        let shipping_borne = req.shipping_borne_by.unwrap_or(ShippingBorneBy::Pembeli);
        let calc_input = SaleCalcInput {
            quantity: req.quantity.unwrap_or(Decimal::ZERO),
            requested_quantity: req.requested_quantity,
            price_per_unit: req.price_per_unit,
            discount: req.discount,
            shipping_cost: req.shipping_cost,
            shipping_borne_by: shipping_borne,
            status,
        };

        let financials = calculate_sale_financials(&calc_input)?;
        let sale_date = req.sale_date.unwrap_or_else(today_wib);

        self.repo
            .update(id, req, financials, sale_date, actor)
            .await
    }

    async fn delete_sale(&self, id: &str) -> Result<(), AppError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_sale_simulasi_2_financials_subsidy_and_fulfillment() {
        // Input Simulasi 2 PRD:
        // Q_req = 50.00 Kg, Q = 45.00 Kg, P = 30000.00, D = 50000.00, S = 40000.00, ShippingBy = "Penjual"
        let input = SaleCalcInput {
            quantity: d("45.00"),
            requested_quantity: Some(d("50.00")),
            price_per_unit: d("30000.00"),
            discount: Some(d("50000.00")),
            shipping_cost: Some(d("40000.00")),
            shipping_borne_by: ShippingBorneBy::Penjual,
            status: SaleStatus::Realized,
        };

        let res = calculate_sale_financials(&input).expect("Calculation must succeed");

        assert_eq!(res.gross_revenue, d("1350000.00"));
        assert_eq!(res.discount, d("50000.00"));
        assert_eq!(res.total_revenue, d("1300000.00"));
        assert_eq!(res.shipping_cost, d("40000.00"));
        assert_eq!(res.seller_shipping_subsidy, d("40000.00"));
        assert_eq!(res.net_proceeds, d("1260000.00"));
        assert_eq!(res.fulfillment_rate_percent, d("90.00"));
        assert_eq!(res.requested_quantity, d("50.00"));
        assert_eq!(res.quantity, d("45.00"));
    }

    #[test]
    fn test_sale_shipping_borne_by_buyer_zero_subsidy() {
        let input = SaleCalcInput {
            quantity: d("10.00"),
            requested_quantity: Some(d("10.00")),
            price_per_unit: d("20000.00"),
            discount: None,
            shipping_cost: Some(d("15000.00")),
            shipping_borne_by: ShippingBorneBy::Pembeli,
            status: SaleStatus::Realized,
        };

        let res = calculate_sale_financials(&input).expect("Calculation must succeed");

        assert_eq!(res.gross_revenue, d("200000.00"));
        assert_eq!(res.discount, d("0.00"));
        assert_eq!(res.total_revenue, d("200000.00"));
        assert_eq!(res.seller_shipping_subsidy, d("0.00"));
        assert_eq!(res.net_proceeds, d("200000.00"));
        assert_eq!(res.fulfillment_rate_percent, d("100.00"));
        assert_eq!(res.requested_quantity, d("10.00"));
        assert_eq!(res.quantity, d("10.00"));
    }

    #[test]
    fn test_discount_greater_than_gross_revenue_rejected() {
        let input = SaleCalcInput {
            quantity: d("5.00"),
            requested_quantity: None,
            price_per_unit: d("10000.00"),
            discount: Some(d("60000.00")), // Gross = 50000 < 60000 -> Err
            shipping_cost: None,
            shipping_borne_by: ShippingBorneBy::Pembeli,
            status: SaleStatus::Realized,
        };

        let err = calculate_sale_financials(&input);
        assert!(err.is_err());
    }

    #[test]
    fn test_sale_order_quantity_zero_uses_requested_quantity_for_revenue() {
        let input = SaleCalcInput {
            quantity: d("0.00"),
            requested_quantity: Some(d("50.00")),
            price_per_unit: d("30000.00"),
            discount: Some(d("50000.00")),
            shipping_cost: Some(d("40000.00")),
            shipping_borne_by: ShippingBorneBy::Penjual,
            status: SaleStatus::Order,
        };

        let res = calculate_sale_financials(&input).expect("Order with qty=0 must succeed");

        // effective_qty = requested_quantity (50.00)
        assert_eq!(res.gross_revenue, d("1500000.00"));
        assert_eq!(res.discount, d("50000.00"));
        assert_eq!(res.total_revenue, d("1450000.00"));
        assert_eq!(res.seller_shipping_subsidy, d("40000.00"));
        assert_eq!(res.net_proceeds, d("1410000.00"));
        assert_eq!(res.fulfillment_rate_percent, d("0.00"));
        assert_eq!(res.requested_quantity, d("50.00"));
        assert_eq!(res.quantity, d("0.00"));
    }

    #[test]
    fn test_sale_order_zero_requested_quantity_rejected() {
        let input = SaleCalcInput {
            quantity: d("0.00"),
            requested_quantity: Some(d("0.00")),
            price_per_unit: d("30000.00"),
            discount: None,
            shipping_cost: None,
            shipping_borne_by: ShippingBorneBy::Pembeli,
            status: SaleStatus::Order,
        };

        let err = calculate_sale_financials(&input);
        assert!(err.is_err());
    }

    #[test]
    fn test_sale_realized_zero_quantity_rejected() {
        let input = SaleCalcInput {
            quantity: d("0.00"),
            requested_quantity: Some(d("50.00")),
            price_per_unit: d("30000.00"),
            discount: None,
            shipping_cost: None,
            shipping_borne_by: ShippingBorneBy::Pembeli,
            status: SaleStatus::Realized,
        };

        let err = calculate_sale_financials(&input);
        assert!(err.is_err());
    }

    #[test]
    fn test_sale_realized_defaults_requested_quantity_to_quantity_when_zero() {
        let input = SaleCalcInput {
            quantity: d("25.00"),
            requested_quantity: None,
            price_per_unit: d("20000.00"),
            discount: None,
            shipping_cost: None,
            shipping_borne_by: ShippingBorneBy::Pembeli,
            status: SaleStatus::Realized,
        };

        let res = calculate_sale_financials(&input).expect("Realized must succeed");

        assert_eq!(res.requested_quantity, d("25.00"));
        assert_eq!(res.quantity, d("25.00"));
        assert_eq!(res.fulfillment_rate_percent, d("100.00"));
        assert_eq!(res.gross_revenue, d("500000.00"));
    }
}
