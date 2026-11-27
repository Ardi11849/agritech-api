use async_trait::async_trait;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use validator::Validate;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::modules::inventory::dto::{
    CreateMovementRequest, CreateStockOpnameRequest, InventoryLedgerResponse,
    InventorySummaryQuery, InventorySummaryResponse, MovementCalculation, MovementType,
    StockCardQuery, StockCardResponse, StockMovementInputType,
};
use crate::modules::inventory::repository::InventoryRepository;

pub fn calculate_stock_movement(
    balance_before: Decimal,
    avg_cost_old: Decimal,
    movement_type: MovementType,
    raw_qty: Decimal,
    raw_unit_cost: Option<Decimal>,
) -> Result<MovementCalculation, AppError> {
    let quantity = round_to_two_decimals(raw_qty);
    if quantity <= Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Kuantitas mutasi harus lebih besar dari 0.00".into(),
        ));
    }

    let max_qty =
        Decimal::from_str("9999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;
    let max_cost =
        Decimal::from_str("999999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;

    if quantity > max_qty {
        return Err(AppError::BadRequest(
            "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
        ));
    }

    let delta_quantity = match movement_type {
        MovementType::StockIn | MovementType::ReturnIn => quantity,
        MovementType::StockOut | MovementType::ReturnOut => -quantity,
        MovementType::OpnameAdjustment => {
            return Err(AppError::BadRequest(
                "Gunakan fungsi stok opname khusus untuk OPNAME_ADJUSTMENT".into(),
            ));
        }
    };

    let balance_after = round_to_two_decimals(balance_before + delta_quantity);
    if balance_after < Decimal::ZERO {
        return Err(AppError::BadRequest(format!(
            "Stok tidak mencukupi. Stok tersedia: {balance_before}, diminta: {quantity}"
        )));
    }

    let (unit_cost, average_unit_cost_after, total_cost_value) = match movement_type {
        MovementType::StockIn => {
            let u_cost = match raw_unit_cost {
                Some(c) => {
                    let rounded = round_to_two_decimals(c);
                    if rounded < Decimal::ZERO {
                        return Err(AppError::BadRequest(
                            "Harga beli per satuan tidak boleh bernilai negatif".into(),
                        ));
                    }
                    if rounded > max_cost {
                        return Err(AppError::BadRequest(
                            "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
                        ));
                    }
                    if quantity * rounded > max_cost {
                        return Err(AppError::BadRequest(
                            "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
                        ));
                    }
                    rounded
                }
                None => Decimal::ZERO,
            };

            let avg_new = if balance_before <= Decimal::ZERO && u_cost > Decimal::ZERO {
                u_cost
            } else if balance_before > Decimal::ZERO && u_cost > Decimal::ZERO {
                let total_val = (balance_before * avg_cost_old) + (quantity * u_cost);
                let total_qty = balance_before + quantity;
                round_to_two_decimals(total_val / total_qty)
            } else {
                avg_cost_old
            };

            let total_cost = round_to_two_decimals(quantity * u_cost);
            (u_cost, avg_new, total_cost)
        }
        MovementType::StockOut | MovementType::ReturnIn | MovementType::ReturnOut => {
            let u_cost = avg_cost_old;
            let avg_new = avg_cost_old;
            let total_cost = round_to_two_decimals(quantity * u_cost);
            (u_cost, avg_new, total_cost)
        }
        MovementType::OpnameAdjustment => unreachable!(),
    };

    Ok(MovementCalculation {
        movement_type,
        quantity,
        delta_quantity,
        balance_before,
        balance_after,
        unit_cost,
        average_unit_cost_after,
        total_cost_value,
    })
}

pub fn calculate_stock_opname(
    balance_before: Decimal,
    avg_cost_old: Decimal,
    raw_physical_stock: Decimal,
) -> Result<MovementCalculation, AppError> {
    if raw_physical_stock < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Hasil hitung fisik tidak boleh bernilai negatif".into(),
        ));
    }

    let max_stock =
        Decimal::from_str("9999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;
    if raw_physical_stock > max_stock {
        return Err(AppError::BadRequest(
            "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
        ));
    }

    let balance_after = round_to_two_decimals(raw_physical_stock);
    let delta_quantity = round_to_two_decimals(balance_after - balance_before);
    let quantity = delta_quantity.abs();
    let unit_cost = avg_cost_old;
    let average_unit_cost_after = avg_cost_old;
    let total_cost_value = round_to_two_decimals(quantity * unit_cost);

    Ok(MovementCalculation {
        movement_type: MovementType::OpnameAdjustment,
        quantity,
        delta_quantity,
        balance_before,
        balance_after,
        unit_cost,
        average_unit_cost_after,
        total_cost_value,
    })
}

pub fn calculate_item_valuation(current_stock: Decimal, average_unit_cost: Decimal) -> Decimal {
    round_to_two_decimals(current_stock * average_unit_cost)
}

#[async_trait]
pub trait InventoryService: Send + Sync {
    async fn get_summary(
        &self,
        query: InventorySummaryQuery,
    ) -> Result<InventorySummaryResponse, AppError>;
    async fn record_movement(
        &self,
        req: CreateMovementRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError>;
    async fn record_stock_opname(
        &self,
        req: CreateStockOpnameRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError>;
    async fn get_stock_card(
        &self,
        item_id: &str,
        query: StockCardQuery,
    ) -> Result<StockCardResponse, AppError>;
}

pub struct InventoryServiceImpl {
    repository: Arc<dyn InventoryRepository>,
}

impl InventoryServiceImpl {
    pub fn new(repository: Arc<dyn InventoryRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl InventoryService for InventoryServiceImpl {
    async fn get_summary(
        &self,
        query: InventorySummaryQuery,
    ) -> Result<InventorySummaryResponse, AppError> {
        self.repository.get_summary(&query).await
    }

    async fn record_movement(
        &self,
        req: CreateMovementRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let max_qty =
            Decimal::from_str("9999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;
        let max_cost =
            Decimal::from_str("999999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;

        if req.quantity > max_qty {
            return Err(AppError::BadRequest(
                "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
            ));
        }
        if let Some(c) = req.unit_cost {
            if c > max_cost {
                return Err(AppError::BadRequest(
                    "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
                ));
            }
            if req.quantity * c > max_cost {
                return Err(AppError::BadRequest(
                    "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
                ));
            }
        }

        // Partner FK combination validation
        match req.movement_type {
            StockMovementInputType::StockIn | StockMovementInputType::ReturnOut => {
                if req.customer_id.is_some() || req.installation_id.is_some() {
                    return Err(AppError::BadRequest(
                        "Tipe mutasi penerimaan/retur keluar hanya dapat berelasi dengan supplier"
                            .into(),
                    ));
                }
                if let Some(sid) = &req.supplier_id {
                    let trimmed = sid.trim();
                    if !trimmed.is_empty() && !self.repository.supplier_exists(trimmed).await? {
                        return Err(AppError::BadRequest(format!(
                            "Supplier dengan ID '{trimmed}' tidak ditemukan"
                        )));
                    }
                }
            }
            StockMovementInputType::StockOut | StockMovementInputType::ReturnIn => {
                if req.supplier_id.is_some() {
                    return Err(AppError::BadRequest(
                        "Tipe mutasi pengeluaran/retur masuk tidak boleh berelasi dengan supplier"
                            .into(),
                    ));
                }
                if req.customer_id.is_some() && req.installation_id.is_some() {
                    return Err(AppError::BadRequest(
                        "Tipe mutasi pengeluaran/retur masuk tidak boleh mengisi customer dan instalasi sekaligus"
                            .into(),
                    ));
                }
                if let Some(cid) = &req.customer_id {
                    let trimmed = cid.trim();
                    if !trimmed.is_empty() && !self.repository.customer_exists(trimmed).await? {
                        return Err(AppError::BadRequest(format!(
                            "Customer dengan ID '{trimmed}' tidak ditemukan"
                        )));
                    }
                }
                if let Some(iid) = &req.installation_id {
                    let trimmed = iid.trim();
                    if !trimmed.is_empty() && !self.repository.installation_exists(trimmed).await? {
                        return Err(AppError::BadRequest(format!(
                            "Instalasi dengan ID '{trimmed}' tidak ditemukan"
                        )));
                    }
                }
            }
        }

        self.repository.record_movement(&req, actor).await
    }

    async fn record_stock_opname(
        &self,
        req: CreateStockOpnameRequest,
        actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let max_qty =
            Decimal::from_str("9999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;
        if req.actual_physical_stock > max_qty {
            return Err(AppError::BadRequest(
                "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
            ));
        }

        let trimmed_notes = req.notes.trim();
        if trimmed_notes.len() < 3 {
            return Err(AppError::BadRequest(
                "Catatan opname minimal 3 karakter".into(),
            ));
        }

        self.repository.record_stock_opname(&req, actor).await
    }

    async fn get_stock_card(
        &self,
        item_id: &str,
        query: StockCardQuery,
    ) -> Result<StockCardResponse, AppError> {
        self.repository.get_stock_card(item_id, &query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::masters::items::dto::StockHealthStatus;
    use crate::modules::masters::items::service::evaluate_stock_health;
    use std::str::FromStr;

    #[test]
    fn test_inventory_movement_and_moving_average_sim_1() {
        let min_stock = Decimal::from_str("10.00").unwrap();
        let mut balance = Decimal::ZERO;
        let mut avg_cost = Decimal::ZERO;

        // Step 1: STOCK_IN 20.00 LTR @ Rp 25.000,00
        let step1 = calculate_stock_movement(
            balance,
            avg_cost,
            MovementType::StockIn,
            Decimal::from_str("20.00").unwrap(),
            Some(Decimal::from_str("25000.00").unwrap()),
        )
        .unwrap();

        assert_eq!(step1.balance_after, Decimal::from_str("20.00").unwrap());
        assert_eq!(
            step1.average_unit_cost_after,
            Decimal::from_str("25000.00").unwrap()
        );
        assert_eq!(
            step1.total_cost_value,
            Decimal::from_str("500000.00").unwrap()
        );
        assert_eq!(
            evaluate_stock_health(step1.balance_after, min_stock),
            StockHealthStatus::Safe
        );

        balance = step1.balance_after;
        avg_cost = step1.average_unit_cost_after;

        // Step 2: STOCK_IN 10.00 LTR @ Rp 31.000,00
        let step2 = calculate_stock_movement(
            balance,
            avg_cost,
            MovementType::StockIn,
            Decimal::from_str("10.00").unwrap(),
            Some(Decimal::from_str("31000.00").unwrap()),
        )
        .unwrap();

        assert_eq!(step2.balance_after, Decimal::from_str("30.00").unwrap());
        assert_eq!(
            step2.average_unit_cost_after,
            Decimal::from_str("27000.00").unwrap()
        );
        assert_eq!(
            evaluate_stock_health(step2.balance_after, min_stock),
            StockHealthStatus::Safe
        );

        balance = step2.balance_after;
        avg_cost = step2.average_unit_cost_after;

        // Step 3: STOCK_OUT 22.50 LTR
        let step3 = calculate_stock_movement(
            balance,
            avg_cost,
            MovementType::StockOut,
            Decimal::from_str("22.50").unwrap(),
            None,
        )
        .unwrap();

        assert_eq!(step3.balance_after, Decimal::from_str("7.50").unwrap());
        assert_eq!(
            step3.average_unit_cost_after,
            Decimal::from_str("27000.00").unwrap()
        );
        assert_eq!(
            step3.total_cost_value,
            Decimal::from_str("607500.00").unwrap()
        );
        assert_eq!(
            evaluate_stock_health(step3.balance_after, min_stock),
            StockHealthStatus::LowStock
        );
    }

    #[test]
    fn test_stock_opname_and_overdraft_prevention_sim_2() {
        let min_stock = Decimal::from_str("10.00").unwrap();
        let balance = Decimal::from_str("7.50").unwrap();
        let avg_cost = Decimal::from_str("27000.00").unwrap();

        // Case A: Stock Opname to 6.75 LTR
        let opname =
            calculate_stock_opname(balance, avg_cost, Decimal::from_str("6.75").unwrap()).unwrap();

        assert_eq!(opname.delta_quantity, Decimal::from_str("-0.75").unwrap());
        assert_eq!(opname.quantity, Decimal::from_str("0.75").unwrap());
        assert_eq!(opname.balance_after, Decimal::from_str("6.75").unwrap());
        assert_eq!(
            opname.total_cost_value,
            Decimal::from_str("20250.00").unwrap()
        );
        assert_eq!(
            evaluate_stock_health(opname.balance_after, min_stock),
            StockHealthStatus::LowStock
        );

        // Case B: Overdraft attempt (try to issue 8.00 when stock is 6.75)
        let new_balance = opname.balance_after;
        let overdraft_err = calculate_stock_movement(
            new_balance,
            avg_cost,
            MovementType::StockOut,
            Decimal::from_str("8.00").unwrap(),
            None,
        );

        assert!(overdraft_err.is_err());
        let err_msg = overdraft_err.err().unwrap().to_string();
        assert!(err_msg.contains("Stok tidak mencukupi"));
        assert!(err_msg.contains("6.75"));
        assert!(err_msg.contains("8.00"));
    }
}
