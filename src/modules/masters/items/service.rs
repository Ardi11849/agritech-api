use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::common::precision::round_to_two_decimals;
use crate::modules::masters::items::dto::{
    CreateMasterItemRequest, ItemListQuery, MasterItemEntity, MasterItemResponse,
    StockHealthStatus, UpdateMasterItemRequest,
};
use crate::modules::masters::items::repository::ItemRepository;

pub fn normalize_unit(unit: Option<&str>) -> Result<String, AppError> {
    match unit {
        None => Ok("PCS".to_string()),
        Some(u) => {
            let trimmed = u.trim();
            if trimmed.is_empty() {
                Ok("PCS".to_string())
            } else if trimmed.len() > 30 {
                Err(AppError::BadRequest(
                    "Satuan dasar maksimal 30 karakter".into(),
                ))
            } else {
                Ok(trimmed.to_uppercase())
            }
        }
    }
}

pub fn validate_and_round_min_stock(val: Decimal) -> Result<Decimal, AppError> {
    if val < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Batas stok minimum tidak boleh bernilai negatif".into(),
        ));
    }
    let max_stock =
        Decimal::from_str("9999999999.99").map_err(|e| AppError::Internal(e.to_string()))?;
    if val > max_stock {
        return Err(AppError::BadRequest(
            "Nilai angka melebihi batas maksimum kapasitas sistem".into(),
        ));
    }
    Ok(round_to_two_decimals(val))
}

pub fn evaluate_stock_health(actual_stock: Decimal, min_stock: Decimal) -> StockHealthStatus {
    if actual_stock <= Decimal::ZERO {
        StockHealthStatus::OutOfStock
    } else if min_stock > Decimal::ZERO && actual_stock <= min_stock {
        StockHealthStatus::LowStock
    } else {
        StockHealthStatus::Safe
    }
}

#[async_trait]
pub trait ItemService: Send + Sync {
    async fn list_items(&self, query: ItemListQuery) -> Result<Vec<MasterItemResponse>, AppError>;
    async fn get_item(&self, id: &str) -> Result<MasterItemResponse, AppError>;
    async fn create_item(
        &self,
        req: CreateMasterItemRequest,
        actor: &str,
    ) -> Result<MasterItemResponse, AppError>;
    async fn update_item(
        &self,
        id: &str,
        req: UpdateMasterItemRequest,
        actor: &str,
    ) -> Result<MasterItemResponse, AppError>;
    async fn delete_item(&self, id: &str) -> Result<(), AppError>;
}

pub struct ItemServiceImpl {
    repository: Arc<dyn ItemRepository>,
}

impl ItemServiceImpl {
    pub fn new(repository: Arc<dyn ItemRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl ItemService for ItemServiceImpl {
    async fn list_items(&self, query: ItemListQuery) -> Result<Vec<MasterItemResponse>, AppError> {
        let rows = self.repository.find_all(&query).await?;
        Ok(rows.into_iter().map(MasterItemResponse::from).collect())
    }

    async fn get_item(&self, id: &str) -> Result<MasterItemResponse, AppError> {
        let row = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item with ID '{id}' not found")))?;
        Ok(MasterItemResponse::from(row))
    }

    async fn create_item(
        &self,
        req: CreateMasterItemRequest,
        actor: &str,
    ) -> Result<MasterItemResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama barang tidak boleh kosong".into(),
            ));
        }

        let category_id = req.category_id.trim().to_string();
        if !self.repository.category_exists(&category_id).await? {
            return Err(AppError::BadRequest(format!(
                "Kategori dengan ID '{category_id}' tidak ditemukan"
            )));
        }

        if self.repository.find_by_name(&name).await?.is_some() {
            return Err(AppError::Conflict(
                "Barang dengan nama tersebut sudah terdaftar".into(),
            ));
        }

        let unit = normalize_unit(req.default_unit.as_deref())?;
        let min_stock = match req.min_stock_level {
            Some(v) => validate_and_round_min_stock(v)?,
            None => Decimal::ZERO,
        };

        let now = Utc::now();
        let entity = MasterItemEntity {
            id: format!("itm_{}", Uuid::new_v4()),
            category_id,
            name,
            default_unit: unit,
            min_stock_level: min_stock,
            current_stock: Decimal::ZERO,
            average_unit_cost: Decimal::ZERO,
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repository.create(entity).await?;
        Ok(MasterItemResponse::from(created))
    }

    async fn update_item(
        &self,
        id: &str,
        req: UpdateMasterItemRequest,
        actor: &str,
    ) -> Result<MasterItemResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let existing = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item with ID '{id}' not found")))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama barang tidak boleh kosong".into(),
            ));
        }

        let category_id = req.category_id.trim().to_string();
        if category_id != existing.category_id
            && !self.repository.category_exists(&category_id).await?
        {
            return Err(AppError::BadRequest(format!(
                "Kategori dengan ID '{category_id}' tidak ditemukan"
            )));
        }

        if !name.eq_ignore_ascii_case(&existing.name) {
            if let Some(dup) = self.repository.find_by_name(&name).await? {
                if dup.id != id {
                    return Err(AppError::Conflict(
                        "Barang dengan nama tersebut sudah terdaftar".into(),
                    ));
                }
            }
        }

        let unit = normalize_unit(req.default_unit.as_deref())?;
        let min_stock = match req.min_stock_level {
            Some(v) => validate_and_round_min_stock(v)?,
            None => Decimal::ZERO,
        };

        let now = Utc::now();
        let entity = MasterItemEntity {
            id: id.to_string(),
            category_id,
            name,
            default_unit: unit,
            min_stock_level: min_stock,
            current_stock: existing.current_stock,
            average_unit_cost: existing.average_unit_cost,
            is_active: req.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let updated = self.repository.update(entity).await?;
        Ok(MasterItemResponse::from(updated))
    }

    async fn delete_item(&self, id: &str) -> Result<(), AppError> {
        let _ = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Item with ID '{id}' not found")))?;

        let ledger_count = self.repository.count_ledgers_by_item(id).await?;
        if ledger_count > 0 {
            return Err(AppError::BadRequest(
                "Barang tidak dapat dihapus karena sudah memiliki riwayat transaksi kartu stok. Nonaktifkan barang (is_active = false) sebagai alternatif."
                    .to_string(),
            ));
        }

        self.repository.delete(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_item_stock_evaluation_and_unit_normalization_sim_1() {
        let unit = normalize_unit(Some(" liter ")).unwrap();
        assert_eq!(unit, "LITER");

        let empty_unit = normalize_unit(Some("   ")).unwrap();
        assert_eq!(empty_unit, "PCS");

        let none_unit = normalize_unit(None).unwrap();
        assert_eq!(none_unit, "PCS");

        let raw_min_stock = Decimal::from_str("5.256").unwrap();
        let rounded = validate_and_round_min_stock(raw_min_stock).unwrap();
        assert_eq!(rounded, Decimal::new(526, 2));

        let neg_stock = Decimal::from_str("-1.00").unwrap();
        assert!(validate_and_round_min_stock(neg_stock).is_err());

        let s_min = rounded;
        assert_eq!(
            evaluate_stock_health(Decimal::from_str("10.00").unwrap(), s_min),
            StockHealthStatus::Safe
        );
        assert_eq!(
            evaluate_stock_health(Decimal::from_str("5.26").unwrap(), s_min),
            StockHealthStatus::LowStock
        );
        assert_eq!(
            evaluate_stock_health(Decimal::from_str("0.00").unwrap(), s_min),
            StockHealthStatus::OutOfStock
        );
        assert_eq!(
            evaluate_stock_health(Decimal::from_str("-1.50").unwrap(), s_min),
            StockHealthStatus::OutOfStock
        );
    }
}
