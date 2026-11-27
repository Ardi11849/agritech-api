use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::masters::locations::dto::{
    CreateLocationRequest, LocationEntity, LocationListQuery, LocationResponse, LocationSummaryRow,
    UpdateLocationRequest,
};
use crate::modules::masters::locations::repository::LocationRepository;

fn map_summary_to_response(r: LocationSummaryRow) -> LocationResponse {
    LocationResponse {
        id: r.id,
        name: r.name,
        manager_name: r.manager_name,
        address: r.address,
        is_active: r.is_active,
        installation_count: r.installation_count,
        active_installation_count: r.active_installation_count,
        total_capacity_holes: r.total_capacity_holes,
        active_capacity_holes: r.active_capacity_holes,
        created_at: r.created_at,
        created_by: r.created_by,
        modify_at: r.modify_at,
        modify_by: r.modify_by,
    }
}

#[async_trait]
pub trait LocationService: Send + Sync {
    async fn list_locations(
        &self,
        query: LocationListQuery,
    ) -> Result<Vec<LocationResponse>, AppError>;
    async fn get_location(&self, id: &str) -> Result<LocationResponse, AppError>;
    async fn create_location(
        &self,
        req: CreateLocationRequest,
        actor: &str,
    ) -> Result<LocationResponse, AppError>;
    async fn update_location(
        &self,
        id: &str,
        req: UpdateLocationRequest,
        actor: &str,
    ) -> Result<LocationResponse, AppError>;
    async fn delete_location(&self, id: &str) -> Result<(), AppError>;
}

pub struct LocationServiceImpl {
    repo: Arc<dyn LocationRepository>,
}

impl LocationServiceImpl {
    pub fn new(repo: Arc<dyn LocationRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl LocationService for LocationServiceImpl {
    async fn list_locations(
        &self,
        query: LocationListQuery,
    ) -> Result<Vec<LocationResponse>, AppError> {
        let rows = self.repo.find_all(&query).await?;
        Ok(rows.into_iter().map(map_summary_to_response).collect())
    }

    async fn get_location(&self, id: &str) -> Result<LocationResponse, AppError> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Location with ID '{id}' not found")))?;
        Ok(map_summary_to_response(row))
    }

    async fn create_location(
        &self,
        req: CreateLocationRequest,
        actor: &str,
    ) -> Result<LocationResponse, AppError> {
        req.validate()?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama lokasi tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }
        if let Some(existing) = self.repo.find_by_name(&name).await? {
            return Err(AppError::Conflict(format!(
                "Lokasi dengan nama '{}' sudah terdaftar",
                existing.name
            )));
        }

        let now = Utc::now();
        let loc_id = format!("loc_{}", Uuid::new_v4());

        let entity = LocationEntity {
            id: loc_id.clone(),
            name,
            manager_name: req.manager_name.unwrap_or_default().trim().to_string(),
            address: req.address.unwrap_or_default().trim().to_string(),
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repo.create(&entity).await?;

        Ok(LocationResponse {
            id: created.id,
            name: created.name,
            manager_name: created.manager_name,
            address: created.address,
            is_active: created.is_active,
            installation_count: 0,
            active_installation_count: 0,
            total_capacity_holes: 0,
            active_capacity_holes: 0,
            created_at: created.created_at,
            created_by: created.created_by,
            modify_at: created.modify_at,
            modify_by: created.modify_by,
        })
    }

    async fn update_location(
        &self,
        id: &str,
        req: UpdateLocationRequest,
        actor: &str,
    ) -> Result<LocationResponse, AppError> {
        req.validate()?;

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Location with ID '{id}' not found")))?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama lokasi tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }
        if let Some(conflict) = self.repo.find_by_name(&name).await? {
            if conflict.id != id {
                return Err(AppError::Conflict(format!(
                    "Lokasi dengan nama '{}' sudah terdaftar",
                    conflict.name
                )));
            }
        }

        let entity = LocationEntity {
            id: id.to_string(),
            name,
            manager_name: req.manager_name.unwrap_or_default().trim().to_string(),
            address: req.address.unwrap_or_default().trim().to_string(),
            is_active: req.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: Utc::now(),
            modify_by: Some(actor.to_string()),
        };

        self.repo.update(&entity).await?;

        // Re-fetch dengan agregasi terbaru
        let updated_summary = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Location with ID '{id}' not found")))?;

        Ok(map_summary_to_response(updated_summary))
    }

    async fn delete_location(&self, id: &str) -> Result<(), AppError> {
        let _ = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Location with ID '{id}' not found")))?;

        let installation_count = self.repo.count_installations_by_location(id).await?;
        if installation_count > 0 {
            return Err(AppError::BadRequest(
                "Lokasi tidak dapat dihapus karena masih memiliki unit instalasi terdaftar"
                    .to_string(),
            ));
        }

        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLocationRepo {
        installation_count: i64,
    }

    #[async_trait]
    impl LocationRepository for MockLocationRepo {
        async fn find_all(
            &self,
            _query: &LocationListQuery,
        ) -> Result<Vec<LocationSummaryRow>, AppError> {
            Ok(vec![])
        }

        async fn find_by_id(&self, id: &str) -> Result<Option<LocationSummaryRow>, AppError> {
            if id == "loc-1" {
                Ok(Some(LocationSummaryRow {
                    id: "loc-1".to_string(),
                    name: "Greenhouse Lembang 1".to_string(),
                    manager_name: "Budi".to_string(),
                    address: "Lembang".to_string(),
                    is_active: true,
                    created_at: Utc::now(),
                    created_by: None,
                    modify_at: Utc::now(),
                    modify_by: None,
                    installation_count: self.installation_count,
                    active_installation_count: self.installation_count,
                    total_capacity_holes: 1200,
                    active_capacity_holes: 1000,
                }))
            } else {
                Ok(None)
            }
        }

        async fn find_by_name(&self, _name: &str) -> Result<Option<LocationEntity>, AppError> {
            Ok(None)
        }

        async fn count_installations_by_location(
            &self,
            _location_id: &str,
        ) -> Result<i64, AppError> {
            Ok(self.installation_count)
        }

        async fn create(&self, _loc: &LocationEntity) -> Result<LocationEntity, AppError> {
            unreachable!()
        }

        async fn update(&self, _loc: &LocationEntity) -> Result<LocationEntity, AppError> {
            unreachable!()
        }

        async fn delete(&self, _id: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_location_deletion_prevented_when_has_installations() {
        // Lokasi dengan instalasi terdaftar harus ditolak saat dihapus
        let repo_busy = Arc::new(MockLocationRepo {
            installation_count: 2,
        });
        let service_busy = LocationServiceImpl::new(repo_busy);
        let res_busy = service_busy.delete_location("loc-1").await;
        assert!(
            matches!(res_busy, Err(AppError::BadRequest(msg)) if msg.contains("unit instalasi terdaftar"))
        );

        // Lokasi tanpa instalasi dapat dihapus
        let repo_empty = Arc::new(MockLocationRepo {
            installation_count: 0,
        });
        let service_empty = LocationServiceImpl::new(repo_empty);
        let res_empty = service_empty.delete_location("loc-1").await;
        assert!(res_empty.is_ok());
    }
}
