use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::common::validation::validate_and_normalize_phone;
use crate::modules::masters::customers::dto::{
    CreateCustomerRequest, CustomerEntity, CustomerListQuery, CustomerResponse,
    UpdateCustomerRequest,
};
use crate::modules::masters::customers::repository::CustomerRepository;

#[async_trait]
pub trait CustomerService: Send + Sync {
    async fn list_customers(
        &self,
        query: CustomerListQuery,
    ) -> Result<Vec<CustomerResponse>, AppError>;
    async fn get_customer(&self, id: &str) -> Result<CustomerResponse, AppError>;
    async fn create_customer(
        &self,
        req: CreateCustomerRequest,
        actor: &str,
    ) -> Result<CustomerResponse, AppError>;
    async fn update_customer(
        &self,
        id: &str,
        req: UpdateCustomerRequest,
        actor: &str,
    ) -> Result<CustomerResponse, AppError>;
    async fn delete_customer(&self, id: &str) -> Result<(), AppError>;
}

pub struct CustomerServiceImpl {
    repository: Arc<dyn CustomerRepository>,
}

impl CustomerServiceImpl {
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CustomerService for CustomerServiceImpl {
    async fn list_customers(
        &self,
        query: CustomerListQuery,
    ) -> Result<Vec<CustomerResponse>, AppError> {
        let rows = self.repository.find_all(&query).await?;
        Ok(rows.into_iter().map(CustomerResponse::from).collect())
    }

    async fn get_customer(&self, id: &str) -> Result<CustomerResponse, AppError> {
        let row = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Customer with ID '{id}' not found")))?;
        Ok(CustomerResponse::from(row))
    }

    async fn create_customer(
        &self,
        req: CreateCustomerRequest,
        actor: &str,
    ) -> Result<CustomerResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama pelanggan tidak boleh kosong".into(),
            ));
        }

        if self.repository.find_by_name(&name).await?.is_some() {
            return Err(AppError::Conflict(
                "Pelanggan dengan nama tersebut sudah terdaftar".into(),
            ));
        }

        let phone = validate_and_normalize_phone(req.phone.as_deref())?;
        let category = match req.category {
            Some(c) if !c.trim().is_empty() => c.trim().to_string(),
            _ => "End-User".to_string(),
        };
        let contact_person = req.contact_person.unwrap_or_default().trim().to_string();
        let address = req.address.unwrap_or_default().trim().to_string();

        let now = Utc::now();
        let entity = CustomerEntity {
            id: format!("cus_{}", Uuid::new_v4()),
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
        Ok(CustomerResponse::from(created))
    }

    async fn update_customer(
        &self,
        id: &str,
        req: UpdateCustomerRequest,
        actor: &str,
    ) -> Result<CustomerResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let existing = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Customer with ID '{id}' not found")))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama pelanggan tidak boleh kosong".into(),
            ));
        }

        if !name.eq_ignore_ascii_case(&existing.name) {
            if let Some(dup) = self.repository.find_by_name(&name).await? {
                if dup.id != id {
                    return Err(AppError::Conflict(
                        "Pelanggan dengan nama tersebut sudah terdaftar".into(),
                    ));
                }
            }
        }

        let phone = validate_and_normalize_phone(req.phone.as_deref())?;
        let category = match req.category {
            Some(c) if !c.trim().is_empty() => c.trim().to_string(),
            _ => "End-User".to_string(),
        };
        let contact_person = req.contact_person.unwrap_or_default().trim().to_string();
        let address = req.address.unwrap_or_default().trim().to_string();

        let now = Utc::now();
        let entity = CustomerEntity {
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
        Ok(CustomerResponse::from(updated))
    }

    async fn delete_customer(&self, id: &str) -> Result<(), AppError> {
        let _ = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Customer with ID '{id}' not found")))?;

        self.repository.delete(id).await?;
        Ok(())
    }
}
