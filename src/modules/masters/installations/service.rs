use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::masters::installations::dto::{
    CreateInstallationRequest, InstallationEntity, InstallationListQuery, InstallationResponse,
    InstallationRow, InstallationStatus, UpdateInstallationRequest,
};
use crate::modules::masters::installations::repository::InstallationRepository;

pub fn calculate_location_capacities(items: &[(i32, InstallationStatus)]) -> (i64, i64) {
    let mut total_holes: i64 = 0;
    let mut active_holes: i64 = 0;

    for (cap, status) in items {
        let holes = *cap as i64;
        total_holes += holes;
        if *status == InstallationStatus::Active {
            active_holes += holes;
        }
    }

    (total_holes, active_holes)
}

pub fn calculate_estimated_yield_kg(
    capacity_holes: i64,
    target_yield_per_hole_gr: Decimal,
) -> Result<Decimal, AppError> {
    if capacity_holes < 0 {
        return Err(AppError::BadRequest(
            "Kapasitas lubang tidak boleh negatif".to_string(),
        ));
    }
    if target_yield_per_hole_gr < Decimal::ZERO {
        return Err(AppError::BadRequest(
            "Target hasil per lubang tidak boleh negatif".to_string(),
        ));
    }

    let total_gr = Decimal::from(capacity_holes) * target_yield_per_hole_gr;
    let kg = total_gr / Decimal::from(1000);
    Ok(kg.round_dp(2))
}

fn map_row_to_response(r: InstallationRow) -> InstallationResponse {
    let status = InstallationStatus::from_str(&r.status).unwrap_or(InstallationStatus::Active);
    InstallationResponse {
        id: r.id,
        location_id: r.location_id,
        location_name: r.location_name,
        name: r.name,
        capacity_holes: r.capacity_holes,
        status,
        notes: r.notes,
        created_at: r.created_at,
        created_by: r.created_by,
        modify_at: r.modify_at,
        modify_by: r.modify_by,
    }
}

#[async_trait]
pub trait InstallationService: Send + Sync {
    async fn list_installations(
        &self,
        query: InstallationListQuery,
    ) -> Result<Vec<InstallationResponse>, AppError>;
    async fn get_installation(&self, id: &str) -> Result<InstallationResponse, AppError>;
    async fn create_installation(
        &self,
        req: CreateInstallationRequest,
        actor: &str,
    ) -> Result<InstallationResponse, AppError>;
    async fn update_installation(
        &self,
        id: &str,
        req: UpdateInstallationRequest,
        actor: &str,
    ) -> Result<InstallationResponse, AppError>;
    async fn delete_installation(&self, id: &str) -> Result<(), AppError>;
}

pub struct InstallationServiceImpl {
    repo: Arc<dyn InstallationRepository>,
}

impl InstallationServiceImpl {
    pub fn new(repo: Arc<dyn InstallationRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl InstallationService for InstallationServiceImpl {
    async fn list_installations(
        &self,
        query: InstallationListQuery,
    ) -> Result<Vec<InstallationResponse>, AppError> {
        let rows = self.repo.find_all(&query).await?;
        Ok(rows.into_iter().map(map_row_to_response).collect())
    }

    async fn get_installation(&self, id: &str) -> Result<InstallationResponse, AppError> {
        let row =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Installation with ID '{id}' not found"))
            })?;
        Ok(map_row_to_response(row))
    }

    async fn create_installation(
        &self,
        req: CreateInstallationRequest,
        actor: &str,
    ) -> Result<InstallationResponse, AppError> {
        req.validate()?;

        if req.capacity_holes < 1 {
            return Err(AppError::BadRequest(
                "Kapasitas lubang tanam minimal 1 lubang".to_string(),
            ));
        }

        let location_name = self
            .repo
            .get_location_name(&req.location_id)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "Lokasi kebun dengan ID '{}' tidak ditemukan",
                    req.location_id
                ))
            })?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama instalasi tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }
        if let Some(existing) = self
            .repo
            .find_by_location_and_name(&req.location_id, &name)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "Unit instalasi dengan nama '{}' sudah terdaftar pada lokasi ini",
                existing.name
            )));
        }

        let now = Utc::now();
        let ins_id = format!("ins_{}", Uuid::new_v4());
        let status = req.status.unwrap_or(InstallationStatus::Active);

        let entity = InstallationEntity {
            id: ins_id.clone(),
            location_id: req.location_id.clone(),
            name: name.clone(),
            capacity_holes: req.capacity_holes,
            status: status.to_string(),
            notes: req.notes.unwrap_or_default().trim().to_string(),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repo.create(&entity).await?;

        Ok(InstallationResponse {
            id: created.id,
            location_id: created.location_id,
            location_name,
            name: created.name,
            capacity_holes: created.capacity_holes,
            status,
            notes: created.notes,
            created_at: created.created_at,
            created_by: created.created_by,
            modify_at: created.modify_at,
            modify_by: created.modify_by,
        })
    }

    async fn update_installation(
        &self,
        id: &str,
        req: UpdateInstallationRequest,
        actor: &str,
    ) -> Result<InstallationResponse, AppError> {
        req.validate()?;

        if req.capacity_holes < 1 {
            return Err(AppError::BadRequest(
                "Kapasitas lubang tanam minimal 1 lubang".to_string(),
            ));
        }

        let existing =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Installation with ID '{id}' not found"))
            })?;

        let location_name = self
            .repo
            .get_location_name(&req.location_id)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "Lokasi kebun dengan ID '{}' tidak ditemukan",
                    req.location_id
                ))
            })?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama instalasi tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }
        if let Some(conflict) = self
            .repo
            .find_by_location_and_name(&req.location_id, &name)
            .await?
        {
            if conflict.id != id {
                return Err(AppError::Conflict(format!(
                    "Unit instalasi dengan nama '{}' sudah terdaftar pada lokasi ini",
                    conflict.name
                )));
            }
        }

        let status = req.status.unwrap_or_else(|| {
            InstallationStatus::from_str(&existing.status).unwrap_or(InstallationStatus::Active)
        });

        let entity = InstallationEntity {
            id: id.to_string(),
            location_id: req.location_id,
            name,
            capacity_holes: req.capacity_holes,
            status: status.to_string(),
            notes: req.notes.unwrap_or_default().trim().to_string(),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: Utc::now(),
            modify_by: Some(actor.to_string()),
        };

        let updated = self.repo.update(&entity).await?;

        Ok(InstallationResponse {
            id: updated.id,
            location_id: updated.location_id,
            location_name,
            name: updated.name,
            capacity_holes: updated.capacity_holes,
            status,
            notes: updated.notes,
            created_at: updated.created_at,
            created_by: updated.created_by,
            modify_at: updated.modify_at,
            modify_by: updated.modify_by,
        })
    }

    async fn delete_installation(&self, id: &str) -> Result<(), AppError> {
        let _ =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Installation with ID '{id}' not found"))
            })?;

        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_installation_capacity_and_yield_sim_2() {
        // Simulasi 2 dari PRD Section 3.5:
        // Lokasi "Greenhouse Lembang 1" memiliki 3 instalasi:
        // - Modul A1: 400 holes, Active
        // - Modul A2: 600 holes, Active
        // - Modul A3: 200 holes, Maintenance
        let items = vec![
            (400, InstallationStatus::Active),
            (600, InstallationStatus::Active),
            (200, InstallationStatus::Maintenance),
        ];

        let (total_holes, active_holes) = calculate_location_capacities(&items);
        assert_eq!(total_holes, 1200);
        assert_eq!(active_holes, 1000);

        // Tanaman Selada Romaine: target_yield_per_hole = 250.00 gr
        let target_yield = Decimal::from_str("250.00").unwrap();
        let yield_kg = calculate_estimated_yield_kg(active_holes, target_yield)
            .expect("Yield calculation should succeed");

        // 1000 lubang * 250 gr / 1000 = 250.00 kg
        assert_eq!(yield_kg, Decimal::from_str("250.00").unwrap());
    }
}
