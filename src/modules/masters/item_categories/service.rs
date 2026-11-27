use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::masters::item_categories::dto::{
    CreateItemCategoryRequest, ItemCategoryEntity, ItemCategoryListQuery, ItemCategoryResponse,
    ItemCategorySummaryRow, UpdateItemCategoryRequest,
};
use crate::modules::masters::item_categories::repository::ItemCategoryRepository;

fn map_summary_to_response(r: ItemCategorySummaryRow) -> ItemCategoryResponse {
    ItemCategoryResponse {
        id: r.id,
        name: r.name,
        description: r.description,
        is_active: r.is_active,
        item_count: r.item_count,
        active_item_count: r.active_item_count,
        created_at: r.created_at,
        created_by: r.created_by,
        modify_at: r.modify_at,
        modify_by: r.modify_by,
    }
}

#[async_trait]
pub trait ItemCategoryService: Send + Sync {
    async fn list_categories(
        &self,
        query: ItemCategoryListQuery,
    ) -> Result<Vec<ItemCategoryResponse>, AppError>;
    async fn get_category(&self, id: &str) -> Result<ItemCategoryResponse, AppError>;
    async fn create_category(
        &self,
        req: CreateItemCategoryRequest,
        actor: &str,
    ) -> Result<ItemCategoryResponse, AppError>;
    async fn update_category(
        &self,
        id: &str,
        req: UpdateItemCategoryRequest,
        actor: &str,
    ) -> Result<ItemCategoryResponse, AppError>;
    async fn delete_category(&self, id: &str) -> Result<(), AppError>;
}

pub struct ItemCategoryServiceImpl {
    repo: Arc<dyn ItemCategoryRepository>,
}

impl ItemCategoryServiceImpl {
    pub fn new(repo: Arc<dyn ItemCategoryRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ItemCategoryService for ItemCategoryServiceImpl {
    async fn list_categories(
        &self,
        query: ItemCategoryListQuery,
    ) -> Result<Vec<ItemCategoryResponse>, AppError> {
        let rows = self.repo.find_all(&query).await?;
        Ok(rows.into_iter().map(map_summary_to_response).collect())
    }

    async fn get_category(&self, id: &str) -> Result<ItemCategoryResponse, AppError> {
        let row =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Item category with ID '{id}' not found"))
            })?;
        Ok(map_summary_to_response(row))
    }

    async fn create_category(
        &self,
        req: CreateItemCategoryRequest,
        actor: &str,
    ) -> Result<ItemCategoryResponse, AppError> {
        req.validate()?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama kategori barang tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }
        if let Some(existing) = self.repo.find_by_name(&name).await? {
            return Err(AppError::Conflict(format!(
                "Kategori barang dengan nama '{}' sudah terdaftar",
                existing.name
            )));
        }

        let now = Utc::now();
        let cat_id = format!("cat_{}", Uuid::new_v4());

        let entity = ItemCategoryEntity {
            id: cat_id.clone(),
            name,
            description: req.description.unwrap_or_default().trim().to_string(),
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repo.create(&entity).await?;

        Ok(ItemCategoryResponse {
            id: created.id,
            name: created.name,
            description: created.description,
            is_active: created.is_active,
            item_count: 0,
            active_item_count: 0,
            created_at: created.created_at,
            created_by: created.created_by,
            modify_at: created.modify_at,
            modify_by: created.modify_by,
        })
    }

    async fn update_category(
        &self,
        id: &str,
        req: UpdateItemCategoryRequest,
        actor: &str,
    ) -> Result<ItemCategoryResponse, AppError> {
        req.validate()?;

        let existing =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Item category with ID '{id}' not found"))
            })?;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama kategori barang tidak boleh kosong atau hanya berisi spasi".to_string(),
            ));
        }
        if let Some(conflict) = self.repo.find_by_name(&name).await? {
            if conflict.id != id {
                return Err(AppError::Conflict(format!(
                    "Kategori barang dengan nama '{}' sudah terdaftar",
                    conflict.name
                )));
            }
        }

        let entity = ItemCategoryEntity {
            id: id.to_string(),
            name,
            description: req.description.unwrap_or_default().trim().to_string(),
            is_active: req.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: Utc::now(),
            modify_by: Some(actor.to_string()),
        };

        self.repo.update(&entity).await?;

        let updated_summary =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Item category with ID '{id}' not found"))
            })?;

        Ok(map_summary_to_response(updated_summary))
    }

    async fn delete_category(&self, id: &str) -> Result<(), AppError> {
        let _ =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Item category with ID '{id}' not found"))
            })?;

        let count = self.repo.count_items_by_category(id).await?;
        if count > 0 {
            return Err(AppError::BadRequest(format!(
                "Kategori tidak dapat dihapus karena masih memiliki {count} barang terdaftar"
            )));
        }

        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockItemCategoryRepo {
        item_count: i64,
    }

    #[async_trait]
    impl ItemCategoryRepository for MockItemCategoryRepo {
        async fn find_all(
            &self,
            _query: &ItemCategoryListQuery,
        ) -> Result<Vec<ItemCategorySummaryRow>, AppError> {
            Ok(vec![])
        }

        async fn find_by_id(&self, id: &str) -> Result<Option<ItemCategorySummaryRow>, AppError> {
            if id == "cat-1" {
                Ok(Some(ItemCategorySummaryRow {
                    id: "cat-1".to_string(),
                    name: "Nutrisi Hidroponik".to_string(),
                    description: "Pekatan AB Mix".to_string(),
                    is_active: true,
                    created_at: Utc::now(),
                    created_by: None,
                    modify_at: Utc::now(),
                    modify_by: None,
                    item_count: self.item_count,
                    active_item_count: self.item_count,
                }))
            } else {
                Ok(None)
            }
        }

        async fn find_by_name(&self, _name: &str) -> Result<Option<ItemCategoryEntity>, AppError> {
            Ok(None)
        }

        async fn count_items_by_category(&self, _category_id: &str) -> Result<i64, AppError> {
            Ok(self.item_count)
        }

        async fn create(&self, _cat: &ItemCategoryEntity) -> Result<ItemCategoryEntity, AppError> {
            unreachable!()
        }

        async fn update(&self, _cat: &ItemCategoryEntity) -> Result<ItemCategoryEntity, AppError> {
            unreachable!()
        }

        async fn delete(&self, _id: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_item_category_deletion_prevented_when_has_items() {
        // Simulasi 2 Kasus B dari PRD Section 3.3:
        // Kategori "Nutrisi Hidroponik" memiliki item_count = 3
        let repo_busy = Arc::new(MockItemCategoryRepo { item_count: 3 });
        let service_busy = ItemCategoryServiceImpl::new(repo_busy);
        let res_busy = service_busy.delete_category("cat-1").await;
        assert!(matches!(
            res_busy,
            Err(AppError::BadRequest(msg)) if msg.contains("masih memiliki 3 barang terdaftar")
        ));

        // Kategori tanpa barang (item_count = 0) dapat dihapus
        let repo_empty = Arc::new(MockItemCategoryRepo { item_count: 0 });
        let service_empty = ItemCategoryServiceImpl::new(repo_empty);
        let res_empty = service_empty.delete_category("cat-1").await;
        assert!(res_empty.is_ok());
    }
}
