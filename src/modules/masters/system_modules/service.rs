use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::system_modules::dto::{
    CreateGroupRequest, CreateModuleRequest, GroupEntity, GroupResponse, ModuleEntity,
    ModuleTreeResponse, UpdateGroupRequest, UpdateModuleRequest,
};
use crate::modules::system_modules::repository::SystemModulesRepository;

pub fn calculate_module_level(parent_id: Option<&str>) -> u8 {
    match parent_id {
        None => 1,
        Some(id) if id.trim().is_empty() => 1,
        Some(_) => 2,
    }
}

#[async_trait]
pub trait SystemModulesService: Send + Sync {
    async fn get_module_groups(&self) -> Result<Vec<GroupResponse>, AppError>;
    async fn get_group_by_id(&self, id: &str) -> Result<GroupResponse, AppError>;
    async fn create_group(&self, req: CreateGroupRequest) -> Result<GroupResponse, AppError>;
    async fn update_group(
        &self,
        id: &str,
        req: UpdateGroupRequest,
    ) -> Result<GroupResponse, AppError>;
    async fn delete_group(&self, id: &str) -> Result<(), AppError>;

    async fn get_all_modules(&self) -> Result<Vec<ModuleTreeResponse>, AppError>;
    async fn get_module_by_id(&self, id: &str) -> Result<ModuleTreeResponse, AppError>;
    async fn create_module(&self, req: CreateModuleRequest)
        -> Result<ModuleTreeResponse, AppError>;
    async fn update_module(
        &self,
        id: &str,
        req: UpdateModuleRequest,
    ) -> Result<ModuleTreeResponse, AppError>;
    async fn delete_module(&self, id: &str) -> Result<(), AppError>;
}

pub struct SystemModulesServiceImpl {
    repo: Arc<dyn SystemModulesRepository>,
}

impl SystemModulesServiceImpl {
    pub fn new(repo: Arc<dyn SystemModulesRepository>) -> Self {
        Self { repo }
    }

    fn assemble_module_tree(
        all_modules: &[ModuleEntity],
        parent_id: Option<&str>,
    ) -> Vec<ModuleTreeResponse> {
        let mut list = Vec::new();

        for m in all_modules {
            let matches_parent = match (&m.parent_id, parent_id) {
                (None, None) => true,
                (Some(p), Some(target)) => p == target,
                _ => false,
            };

            if matches_parent {
                let sub_modules = Self::assemble_module_tree(all_modules, Some(&m.id));
                list.push(ModuleTreeResponse {
                    id: m.id.clone(),
                    group_id: m.group_id.clone(),
                    parent_id: m.parent_id.clone(),
                    name: m.name.clone(),
                    path: m.path.clone(),
                    icon: m.icon.clone().unwrap_or_default(),
                    is_active: m.is_active,
                    created_at: m.created_at,
                    created_by: m.created_by.clone().unwrap_or_else(|| "system".to_string()),
                    modify_at: m.modify_at,
                    modify_by: m.modify_by.clone().unwrap_or_else(|| "system".to_string()),
                    sub_modules,
                });
            }
        }

        list
    }
}

#[async_trait]
impl SystemModulesService for SystemModulesServiceImpl {
    async fn get_module_groups(&self) -> Result<Vec<GroupResponse>, AppError> {
        let groups = self.repo.find_all_groups().await?;
        let all_modules = self.repo.find_all_modules().await?;

        let mut result = Vec::new();
        for g in groups {
            let group_modules: Vec<ModuleEntity> = all_modules
                .iter()
                .filter(|m| m.group_id == g.id)
                .cloned()
                .collect();

            let root_modules = Self::assemble_module_tree(&group_modules, None);

            result.push(GroupResponse {
                id: g.id,
                name: g.name,
                icon: g.icon.unwrap_or_default(),
                is_active: g.is_active,
                created_at: g.created_at,
                modify_at: g.modify_at,
                modules: root_modules,
            });
        }

        Ok(result)
    }

    async fn get_group_by_id(&self, id: &str) -> Result<GroupResponse, AppError> {
        let group =
            self.repo.find_group_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Module group with ID '{id}' not found"))
            })?;
        let modules = self.repo.find_modules_by_group(id).await?;
        let module_tree = Self::assemble_module_tree(&modules, None);

        Ok(GroupResponse {
            id: group.id,
            name: group.name,
            icon: group.icon.unwrap_or_default(),
            is_active: group.is_active,
            created_at: group.created_at,
            modify_at: group.modify_at,
            modules: module_tree,
        })
    }

    async fn create_group(&self, req: CreateGroupRequest) -> Result<GroupResponse, AppError> {
        req.validate()?;
        let name = crate::common::validation::validate_trimmed_non_empty(
            &req.name,
            "Nama grup modul",
            100,
        )?;
        let icon = match req.icon {
            Some(i) => {
                let trimmed = i.trim().to_string();
                if trimmed.len() > 100 {
                    return Err(AppError::BadRequest("Icon maksimal 100 karakter".into()));
                }
                Some(trimmed)
            }
            None => None,
        };
        let now = Utc::now();
        let group_id = format!("grp-{}", Uuid::new_v4());

        let entity = GroupEntity {
            id: group_id,
            name,
            icon,
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some("system".to_string()),
            modify_at: now,
            modify_by: Some("system".to_string()),
        };

        let created = self.repo.create_group(&entity).await?;

        Ok(GroupResponse {
            id: created.id,
            name: created.name,
            icon: created.icon.unwrap_or_default(),
            is_active: created.is_active,
            created_at: created.created_at,
            modify_at: created.modify_at,
            modules: Vec::new(),
        })
    }

    async fn update_group(
        &self,
        id: &str,
        req: UpdateGroupRequest,
    ) -> Result<GroupResponse, AppError> {
        req.validate()?;
        let mut group =
            self.repo.find_group_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Module group with ID '{id}' not found"))
            })?;

        if let Some(name) = req.name {
            group.name = crate::common::validation::validate_trimmed_non_empty(
                &name,
                "Nama grup modul",
                100,
            )?;
        }
        if let Some(icon) = req.icon {
            let trimmed = icon.trim().to_string();
            if trimmed.len() > 100 {
                return Err(AppError::BadRequest("Icon maksimal 100 karakter".into()));
            }
            group.icon = Some(trimmed);
        }
        if let Some(active) = req.is_active {
            group.is_active = active;
        }
        group.modify_at = Utc::now();
        group.modify_by = Some("system".to_string());

        let updated = self.repo.update_group(&group).await?;
        let modules = self.repo.find_modules_by_group(id).await?;
        let module_tree = Self::assemble_module_tree(&modules, None);

        Ok(GroupResponse {
            id: updated.id,
            name: updated.name,
            icon: updated.icon.unwrap_or_default(),
            is_active: updated.is_active,
            created_at: updated.created_at,
            modify_at: updated.modify_at,
            modules: module_tree,
        })
    }

    async fn delete_group(&self, id: &str) -> Result<(), AppError> {
        self.repo.delete_group(id).await
    }

    async fn create_module(
        &self,
        req: CreateModuleRequest,
    ) -> Result<ModuleTreeResponse, AppError> {
        req.validate()?;
        let name =
            crate::common::validation::validate_trimmed_non_empty(&req.name, "Nama modul", 100)?;
        let path =
            crate::common::validation::validate_trimmed_non_empty(&req.path, "Path modul", 100)?;
        let icon = match req.icon {
            Some(i) => {
                let trimmed = i.trim().to_string();
                if trimmed.len() > 100 {
                    return Err(AppError::BadRequest("Icon maksimal 100 karakter".into()));
                }
                Some(trimmed)
            }
            None => None,
        };

        if self.repo.find_group_by_id(&req.group_id).await?.is_none() {
            return Err(AppError::BadRequest(format!(
                "Group with ID '{}' not found",
                req.group_id
            )));
        }

        if let Some(ref p_id) = req.parent_id {
            if self.repo.find_module_by_id(p_id).await?.is_none() {
                return Err(AppError::BadRequest(format!(
                    "Parent module with ID '{p_id}' not found"
                )));
            }
        }

        let now = Utc::now();
        let module_id = format!("mod-{}", Uuid::new_v4());

        let entity = ModuleEntity {
            id: module_id,
            group_id: req.group_id,
            parent_id: req.parent_id,
            name,
            path,
            icon,
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some("system".to_string()),
            modify_at: now,
            modify_by: Some("system".to_string()),
        };

        let created = self.repo.create_module(&entity).await?;

        Ok(ModuleTreeResponse {
            id: created.id,
            group_id: created.group_id,
            parent_id: created.parent_id,
            name: created.name,
            path: created.path,
            icon: created.icon.unwrap_or_default(),
            is_active: created.is_active,
            created_at: created.created_at,
            created_by: created.created_by.unwrap_or_else(|| "system".to_string()),
            modify_at: created.modify_at,
            modify_by: created.modify_by.unwrap_or_else(|| "system".to_string()),
            sub_modules: Vec::new(),
        })
    }

    async fn get_all_modules(&self) -> Result<Vec<ModuleTreeResponse>, AppError> {
        let all_modules = self.repo.find_all_modules().await?;
        Ok(Self::assemble_module_tree(&all_modules, None))
    }

    async fn get_module_by_id(&self, id: &str) -> Result<ModuleTreeResponse, AppError> {
        let module = self
            .repo
            .find_module_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Module with ID '{id}' not found")))?;

        let all_modules = self.repo.find_all_modules().await?;
        let sub_modules = Self::assemble_module_tree(&all_modules, Some(&module.id));

        Ok(ModuleTreeResponse {
            id: module.id,
            group_id: module.group_id,
            parent_id: module.parent_id,
            name: module.name,
            path: module.path,
            icon: module.icon.unwrap_or_default(),
            is_active: module.is_active,
            created_at: module.created_at,
            created_by: module.created_by.unwrap_or_else(|| "system".to_string()),
            modify_at: module.modify_at,
            modify_by: module.modify_by.unwrap_or_else(|| "system".to_string()),
            sub_modules,
        })
    }

    async fn update_module(
        &self,
        id: &str,
        req: UpdateModuleRequest,
    ) -> Result<ModuleTreeResponse, AppError> {
        req.validate()?;

        let mut module = self
            .repo
            .find_module_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Module with ID '{id}' not found")))?;

        if let Some(ref p_id) = req.parent_id {
            if p_id == id {
                return Err(AppError::BadRequest(
                    "Circular dependency detected: Module cannot be its own parent".to_string(),
                ));
            }
            if self.repo.find_module_by_id(p_id).await?.is_none() {
                return Err(AppError::BadRequest(format!(
                    "Parent module with ID '{p_id}' not found"
                )));
            }

            // Check multi-level ancestry: target parent must not be a descendant of this module
            let all_modules = self.repo.find_all_modules().await?;
            let mut parent_map = std::collections::HashMap::new();
            for m in &all_modules {
                parent_map.insert(m.id.clone(), m.parent_id.clone());
            }

            let mut curr = p_id.clone();
            let mut visited = std::collections::HashSet::new();
            visited.insert(curr.clone());

            while let Some(Some(ancestor_id)) = parent_map.get(&curr) {
                if ancestor_id == id {
                    return Err(AppError::BadRequest(
                        "Circular dependency detected: Target parent is a descendant of this module".to_string(),
                    ));
                }
                if !visited.insert(ancestor_id.clone()) {
                    return Err(AppError::BadRequest(
                        "Circular dependency detected in hierarchy".to_string(),
                    ));
                }
                curr = ancestor_id.clone();
            }

            module.parent_id = Some(p_id.clone());
        }

        if let Some(ref g_id) = req.group_id {
            if self.repo.find_group_by_id(g_id).await?.is_none() {
                return Err(AppError::BadRequest(format!(
                    "Group with ID '{g_id}' not found"
                )));
            }
            module.group_id = g_id.clone();
        }

        if let Some(name) = req.name {
            module.name =
                crate::common::validation::validate_trimmed_non_empty(&name, "Nama modul", 100)?;
        }
        if let Some(path) = req.path {
            module.path =
                crate::common::validation::validate_trimmed_non_empty(&path, "Path modul", 100)?;
        }
        if let Some(icon) = req.icon {
            let trimmed = icon.trim().to_string();
            if trimmed.len() > 100 {
                return Err(AppError::BadRequest("Icon maksimal 100 karakter".into()));
            }
            module.icon = Some(trimmed);
        }
        if let Some(active) = req.is_active {
            module.is_active = active;
        }
        module.modify_at = Utc::now();
        module.modify_by = Some("system".to_string());

        let updated = self.repo.update_module(&module).await?;

        Ok(ModuleTreeResponse {
            id: updated.id,
            group_id: updated.group_id,
            parent_id: updated.parent_id,
            name: updated.name,
            path: updated.path,
            icon: updated.icon.unwrap_or_default(),
            is_active: updated.is_active,
            created_at: updated.created_at,
            created_by: updated.created_by.unwrap_or_else(|| "system".to_string()),
            modify_at: updated.modify_at,
            modify_by: updated.modify_by.unwrap_or_else(|| "system".to_string()),
            sub_modules: Vec::new(),
        })
    }

    async fn delete_module(&self, id: &str) -> Result<(), AppError> {
        self.repo.delete_module(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_level_calculation_formula() {
        // Rumus dari PRD 3.2: Level = 1 if parent_id is NULL, 2 if parent_id is NOT NULL
        assert_eq!(calculate_module_level(None), 1);
        assert_eq!(calculate_module_level(Some("")), 1);
        assert_eq!(calculate_module_level(Some("mod-parent-1")), 2);
    }

    #[tokio::test]
    async fn test_circular_dependency_prevention() {
        struct MockModuleRepo {
            module: ModuleEntity,
        }
        #[async_trait]
        impl SystemModulesRepository for MockModuleRepo {
            async fn find_all_groups(&self) -> Result<Vec<GroupEntity>, AppError> {
                Ok(vec![])
            }
            async fn find_group_by_id(&self, _id: &str) -> Result<Option<GroupEntity>, AppError> {
                Ok(None)
            }
            async fn create_group(&self, _group: &GroupEntity) -> Result<GroupEntity, AppError> {
                unreachable!()
            }
            async fn update_group(&self, _group: &GroupEntity) -> Result<GroupEntity, AppError> {
                unreachable!()
            }
            async fn delete_group(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
            async fn find_all_modules(&self) -> Result<Vec<ModuleEntity>, AppError> {
                Ok(vec![self.module.clone()])
            }
            async fn find_modules_by_group(
                &self,
                _group_id: &str,
            ) -> Result<Vec<ModuleEntity>, AppError> {
                Ok(vec![self.module.clone()])
            }
            async fn find_module_by_id(&self, id: &str) -> Result<Option<ModuleEntity>, AppError> {
                if id == self.module.id {
                    Ok(Some(self.module.clone()))
                } else {
                    Ok(None)
                }
            }
            async fn create_module(
                &self,
                _module: &ModuleEntity,
            ) -> Result<ModuleEntity, AppError> {
                unreachable!()
            }
            async fn update_module(
                &self,
                _module: &ModuleEntity,
            ) -> Result<ModuleEntity, AppError> {
                unreachable!()
            }
            async fn delete_module(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
        }

        let repo = Arc::new(MockModuleRepo {
            module: ModuleEntity {
                id: "mod-1".to_string(),
                group_id: "grp-1".to_string(),
                parent_id: None,
                name: "Plant".to_string(),
                path: "/plants".to_string(),
                icon: None,
                is_active: true,
                created_at: Utc::now(),
                created_by: None,
                modify_at: Utc::now(),
                modify_by: None,
            },
        });

        let service = SystemModulesServiceImpl::new(repo);
        let update_req = UpdateModuleRequest {
            group_id: None,
            parent_id: Some("mod-1".to_string()), // Setting parent to itself!
            name: None,
            path: None,
            icon: None,
            is_active: None,
        };

        let result = service.update_module("mod-1", update_req).await;
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_multilevel_circular_dependency_prevention() {
        // Hierarchy: A -> B -> C
        // Updating A's parent to C must fail with Circular Dependency error
        struct MultiLevelMockRepo {
            modules: Vec<ModuleEntity>,
        }
        #[async_trait]
        impl SystemModulesRepository for MultiLevelMockRepo {
            async fn find_all_groups(&self) -> Result<Vec<GroupEntity>, AppError> {
                Ok(vec![])
            }
            async fn find_group_by_id(&self, _id: &str) -> Result<Option<GroupEntity>, AppError> {
                Ok(None)
            }
            async fn create_group(&self, _group: &GroupEntity) -> Result<GroupEntity, AppError> {
                unreachable!()
            }
            async fn update_group(&self, _group: &GroupEntity) -> Result<GroupEntity, AppError> {
                unreachable!()
            }
            async fn delete_group(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
            async fn find_all_modules(&self) -> Result<Vec<ModuleEntity>, AppError> {
                Ok(self.modules.clone())
            }
            async fn find_modules_by_group(
                &self,
                _group_id: &str,
            ) -> Result<Vec<ModuleEntity>, AppError> {
                Ok(self.modules.clone())
            }
            async fn find_module_by_id(&self, id: &str) -> Result<Option<ModuleEntity>, AppError> {
                Ok(self.modules.iter().find(|m| m.id == id).cloned())
            }
            async fn create_module(
                &self,
                _module: &ModuleEntity,
            ) -> Result<ModuleEntity, AppError> {
                unreachable!()
            }
            async fn update_module(
                &self,
                _module: &ModuleEntity,
            ) -> Result<ModuleEntity, AppError> {
                unreachable!()
            }
            async fn delete_module(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
        }

        let modules = vec![
            ModuleEntity {
                id: "mod-A".to_string(),
                group_id: "grp-1".to_string(),
                parent_id: None,
                name: "Mod A".to_string(),
                path: "/a".to_string(),
                icon: None,
                is_active: true,
                created_at: Utc::now(),
                created_by: None,
                modify_at: Utc::now(),
                modify_by: None,
            },
            ModuleEntity {
                id: "mod-B".to_string(),
                group_id: "grp-1".to_string(),
                parent_id: Some("mod-A".to_string()),
                name: "Mod B".to_string(),
                path: "/a/b".to_string(),
                icon: None,
                is_active: true,
                created_at: Utc::now(),
                created_by: None,
                modify_at: Utc::now(),
                modify_by: None,
            },
            ModuleEntity {
                id: "mod-C".to_string(),
                group_id: "grp-1".to_string(),
                parent_id: Some("mod-B".to_string()),
                name: "Mod C".to_string(),
                path: "/a/b/c".to_string(),
                icon: None,
                is_active: true,
                created_at: Utc::now(),
                created_by: None,
                modify_at: Utc::now(),
                modify_by: None,
            },
        ];

        let repo = Arc::new(MultiLevelMockRepo { modules });
        let service = SystemModulesServiceImpl::new(repo);

        // Try setting A's parent to C (descendant cycle!)
        let update_req = UpdateModuleRequest {
            group_id: None,
            parent_id: Some("mod-C".to_string()),
            name: None,
            path: None,
            icon: None,
            is_active: None,
        };

        let result = service.update_module("mod-A", update_req).await;
        assert!(
            matches!(result, Err(AppError::BadRequest(msg)) if msg.contains("Circular dependency"))
        );
    }
}
