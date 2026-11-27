use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::common::validation::validate_and_normalize_phone;
use crate::modules::masters::suppliers::dto::{
    CreateSupplierRequest, SupplierEntity, SupplierListQuery, SupplierResponse,
    UpdateSupplierRequest,
};
use crate::modules::masters::suppliers::repository::SupplierRepository;

#[async_trait]
pub trait SupplierService: Send + Sync {
    async fn list_suppliers(
        &self,
        query: SupplierListQuery,
    ) -> Result<Vec<SupplierResponse>, AppError>;
    async fn get_supplier(&self, id: &str) -> Result<SupplierResponse, AppError>;
    async fn create_supplier(
        &self,
        req: CreateSupplierRequest,
        actor: &str,
    ) -> Result<SupplierResponse, AppError>;
    async fn update_supplier(
        &self,
        id: &str,
        req: UpdateSupplierRequest,
        actor: &str,
    ) -> Result<SupplierResponse, AppError>;
    async fn delete_supplier(&self, id: &str) -> Result<(), AppError>;
}

pub struct SupplierServiceImpl {
    repository: Arc<dyn SupplierRepository>,
}

impl SupplierServiceImpl {
    pub fn new(repository: Arc<dyn SupplierRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl SupplierService for SupplierServiceImpl {
    async fn list_suppliers(
        &self,
        query: SupplierListQuery,
    ) -> Result<Vec<SupplierResponse>, AppError> {
        let rows = self.repository.find_all(&query).await?;
        Ok(rows.into_iter().map(SupplierResponse::from).collect())
    }

    async fn get_supplier(&self, id: &str) -> Result<SupplierResponse, AppError> {
        let row = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Supplier with ID '{id}' not found")))?;
        Ok(SupplierResponse::from(row))
    }

    async fn create_supplier(
        &self,
        req: CreateSupplierRequest,
        actor: &str,
    ) -> Result<SupplierResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama pemasok tidak boleh kosong".into(),
            ));
        }

        if self.repository.find_by_name(&name).await?.is_some() {
            return Err(AppError::Conflict(
                "Pemasok dengan nama tersebut sudah terdaftar".into(),
            ));
        }

        let phone = validate_and_normalize_phone(req.phone.as_deref())?;
        let category = match req.category {
            Some(c) if !c.trim().is_empty() => c.trim().to_string(),
            _ => "Lainnya".to_string(),
        };
        let contact_person = req.contact_person.unwrap_or_default().trim().to_string();
        let address = req.address.unwrap_or_default().trim().to_string();

        let now = Utc::now();
        let entity = SupplierEntity {
            id: format!("sup_{}", Uuid::new_v4()),
            name,
            category,
            contact_person,
            phone,
            address,
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repository.create(entity).await?;
        Ok(SupplierResponse::from(created))
    }

    async fn update_supplier(
        &self,
        id: &str,
        req: UpdateSupplierRequest,
        actor: &str,
    ) -> Result<SupplierResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let existing = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Supplier with ID '{id}' not found")))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama pemasok tidak boleh kosong".into(),
            ));
        }

        if !name.eq_ignore_ascii_case(&existing.name) {
            if let Some(dup) = self.repository.find_by_name(&name).await? {
                if dup.id != id {
                    return Err(AppError::Conflict(
                        "Pemasok dengan nama tersebut sudah terdaftar".into(),
                    ));
                }
            }
        }

        let phone = validate_and_normalize_phone(req.phone.as_deref())?;
        let category = match req.category {
            Some(c) if !c.trim().is_empty() => c.trim().to_string(),
            _ => "Lainnya".to_string(),
        };
        let contact_person = req.contact_person.unwrap_or_default().trim().to_string();
        let address = req.address.unwrap_or_default().trim().to_string();

        let now = Utc::now();
        let entity = SupplierEntity {
            id: id.to_string(),
            name,
            category,
            contact_person,
            phone,
            address,
            is_active: req.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let updated = self.repository.update(entity).await?;
        Ok(SupplierResponse::from(updated))
    }

    async fn delete_supplier(&self, id: &str) -> Result<(), AppError> {
        let _ = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Supplier with ID '{id}' not found")))?;

        self.repository.delete(id).await?;
        Ok(())
    }
}
