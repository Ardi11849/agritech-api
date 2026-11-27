use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::role::dto::{
    CreateRolePermissionRequest, CreateRoleRequest, ModuleDetailDto, ModuleGroupDetailDto,
    RoleDetailResponse, RoleEntity, RoleItemResponse, RolePermissionDto, RolePermissionEntity,
    UpdateRolePermissionRequest, UpdateRoleRequest,
};
use crate::modules::role::repository::{RolePermissionRepository, RoleRepository};
use crate::modules::system_modules::dto::ModuleEntity;

pub fn normalize_permissions(
    can_create: bool,
    can_read: bool,
    can_update: bool,
    can_delete: bool,
) -> (bool, bool, bool, bool) {
    let normalized_read = can_read || can_create || can_update || can_delete;
    (can_create, normalized_read, can_update, can_delete)
}

pub fn is_access_granted(
    role_active: bool,
    group_active: bool,
    module_active: bool,
    can_read: bool,
) -> bool {
    role_active && group_active && module_active && can_read
}

#[async_trait]
pub trait RoleService: Send + Sync {
    async fn get_roles(&self) -> Result<Vec<RoleItemResponse>, AppError>;
    async fn get_role_detail(&self, id: &str) -> Result<RoleDetailResponse, AppError>;
    async fn create_role(&self, req: CreateRoleRequest) -> Result<RoleItemResponse, AppError>;
    async fn update_role(
        &self,
        id: &str,
        req: UpdateRoleRequest,
    ) -> Result<RoleItemResponse, AppError>;
    async fn delete_role(&self, id: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait RolePermissionService: Send + Sync {
    async fn get_all_permissions(&self) -> Result<Vec<RolePermissionDto>, AppError>;
    async fn get_permission_by_id(&self, id: &str) -> Result<RolePermissionDto, AppError>;
    async fn get_permissions_by_role(
        &self,
        role_id: &str,
    ) -> Result<Vec<RolePermissionDto>, AppError>;
    async fn create_permission(
        &self,
        req: CreateRolePermissionRequest,
    ) -> Result<RolePermissionDto, AppError>;
    async fn update_permission(
        &self,
        id: &str,
        req: UpdateRolePermissionRequest,
    ) -> Result<RolePermissionDto, AppError>;
    async fn delete_permission(&self, id: &str) -> Result<(), AppError>;
}

pub struct RoleServiceImpl {
    repo: Arc<dyn RoleRepository>,
}

impl RoleServiceImpl {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    fn assemble_module_tree(
        role_active: bool,
        group_active: bool,
        all_modules: &[ModuleEntity],
        permissions: &[RolePermissionEntity],
        parent_id: Option<&str>,
    ) -> Vec<ModuleDetailDto> {
        let mut list = Vec::new();

        for m in all_modules {
            let matches_parent = match (&m.parent_id, parent_id) {
                (None, None) => true,
                (Some(p), Some(target)) => p == target,
                _ => false,
            };

            if matches_parent {
                let sub_modules = Self::assemble_module_tree(
                    role_active,
                    group_active,
                    all_modules,
                    permissions,
                    Some(&m.id),
                );
                let module_perms: Vec<RolePermissionDto> = permissions
                    .iter()
                    .filter(|p| p.module_id == m.id)
                    .map(|p| RolePermissionDto {
                        id: p.id.clone(),
                        role_id: p.role_id.clone(),
                        module_group_id: p.module_group_id.clone(),
                        module_id: p.module_id.clone(),
                        can_create: p.can_create,
                        can_read: p.can_read,
                        can_update: p.can_update,
                        can_delete: p.can_delete,
                        created_at: p.created_at,
                        modify_at: p.modify_at,
                    })
                    .collect();

                let can_read = module_perms.iter().any(|p| p.can_read);
                let access_granted =
                    is_access_granted(role_active, group_active, m.is_active, can_read);

                if access_granted || !sub_modules.is_empty() {
                    list.push(ModuleDetailDto {
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
                        role_permission: module_perms,
                    });
                }
            }
        }

        list
    }
}

#[async_trait]
impl RoleService for RoleServiceImpl {
    async fn get_roles(&self) -> Result<Vec<RoleItemResponse>, AppError> {
        let roles = self.repo.find_all().await?;
        let res = roles
            .into_iter()
            .map(|r| RoleItemResponse {
                id: r.id,
                name: r.name,
                is_active: r.is_active,
                created_at: r.created_at,
                modify_at: r.modify_at,
            })
            .collect();
        Ok(res)
    }

    async fn get_role_detail(&self, id: &str) -> Result<RoleDetailResponse, AppError> {
        let role = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with ID '{id}' not found")))?;

        let (groups, modules, perms) = self.repo.find_groups_and_modules_for_role(id).await?;

        let mut group_details = Vec::new();
        for g in groups {
            let group_modules: Vec<ModuleEntity> = modules
                .iter()
                .filter(|m| m.group_id == g.id)
                .cloned()
                .collect();

            let root_modules = Self::assemble_module_tree(
                role.is_active,
                g.is_active,
                &group_modules,
                &perms,
                None,
            );

            if g.is_active || !root_modules.is_empty() {
                group_details.push(ModuleGroupDetailDto {
                    id: g.id,
                    name: g.name,
                    icon: g.icon.unwrap_or_default(),
                    is_active: g.is_active,
                    created_at: g.created_at,
                    modify_at: g.modify_at,
                    module: root_modules,
                });
            }
        }

        Ok(RoleDetailResponse {
            id: role.id,
            name: role.name,
            is_active: role.is_active,
            created_at: role.created_at,
            modify_at: role.modify_at,
            module_group: group_details,
        })
    }

    async fn create_role(&self, req: CreateRoleRequest) -> Result<RoleItemResponse, AppError> {
        req.validate()?;
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama role tidak boleh kosong".to_string(),
            ));
        }

        if self.repo.find_by_name(&name).await?.is_some() {
            return Err(AppError::Conflict(format!(
                "Role with name '{}' already exists",
                name
            )));
        }

        let now = Utc::now();
        let role_id = format!("role-{}", Uuid::new_v4());

        let entity = RoleEntity {
            id: role_id,
            name,
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some("system".to_string()),
            modify_at: now,
            modify_by: Some("system".to_string()),
        };

        let created = self.repo.create(&entity).await?;

        Ok(RoleItemResponse {
            id: created.id,
            name: created.name,
            is_active: created.is_active,
            created_at: created.created_at,
            modify_at: created.modify_at,
        })
    }

    async fn update_role(
        &self,
        id: &str,
        req: UpdateRoleRequest,
    ) -> Result<RoleItemResponse, AppError> {
        req.validate()?;
        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Nama role tidak boleh kosong".to_string(),
            ));
        }

        let mut role = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with ID '{id}' not found")))?;

        let is_protected_role = role.name.trim().eq_ignore_ascii_case("ADMINISTRATOR")
            || role.name.trim().eq_ignore_ascii_case("SUPER ADMIN");
        if is_protected_role && req.is_active == Some(false) {
            return Err(AppError::BadRequest(
                "Administrator role cannot be deactivated".to_string(),
            ));
        }

        if let Some(existing) = self.repo.find_by_name(&name).await? {
            if existing.id != id {
                return Err(AppError::Conflict(format!(
                    "Role with name '{}' already exists",
                    name
                )));
            }
        }

        role.name = name;
        if let Some(active) = req.is_active {
            role.is_active = active;
        }
        role.modify_at = Utc::now();
        role.modify_by = Some("system".to_string());

        let updated = self.repo.update(&role).await?;

        Ok(RoleItemResponse {
            id: updated.id,
            name: updated.name,
            is_active: updated.is_active,
            created_at: updated.created_at,
            modify_at: updated.modify_at,
        })
    }

    async fn delete_role(&self, id: &str) -> Result<(), AppError> {
        let role = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with ID '{id}' not found")))?;

        let is_protected_role = role.name.trim().eq_ignore_ascii_case("ADMINISTRATOR")
            || role.name.trim().eq_ignore_ascii_case("SUPER ADMIN");
        if is_protected_role {
            return Err(AppError::BadRequest(
                "Administrator role is protected and cannot be deleted".to_string(),
            ));
        }

        if self.repo.count_users_by_role_id(id).await? > 0 {
            return Err(AppError::BadRequest(
                "Role cannot be deleted because it is still assigned to users".to_string(),
            ));
        }

        self.repo.delete(id).await
    }
}

pub struct RolePermissionServiceImpl {
    repo: Arc<dyn RolePermissionRepository>,
}

impl RolePermissionServiceImpl {
    pub fn new(repo: Arc<dyn RolePermissionRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RolePermissionService for RolePermissionServiceImpl {
    async fn get_all_permissions(&self) -> Result<Vec<RolePermissionDto>, AppError> {
        let perms = self.repo.find_all().await?;
        Ok(perms
            .into_iter()
            .map(|p| RolePermissionDto {
                id: p.id,
                role_id: p.role_id,
                module_group_id: p.module_group_id,
                module_id: p.module_id,
                can_create: p.can_create,
                can_read: p.can_read,
                can_update: p.can_update,
                can_delete: p.can_delete,
                created_at: p.created_at,
                modify_at: p.modify_at,
            })
            .collect())
    }

    async fn get_permission_by_id(&self, id: &str) -> Result<RolePermissionDto, AppError> {
        let p =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Permission with ID '{id}' not found"))
            })?;

        Ok(RolePermissionDto {
            id: p.id,
            role_id: p.role_id,
            module_group_id: p.module_group_id,
            module_id: p.module_id,
            can_create: p.can_create,
            can_read: p.can_read,
            can_update: p.can_update,
            can_delete: p.can_delete,
            created_at: p.created_at,
            modify_at: p.modify_at,
        })
    }

    async fn get_permissions_by_role(
        &self,
        role_id: &str,
    ) -> Result<Vec<RolePermissionDto>, AppError> {
        let perms = self.repo.find_by_role_id(role_id).await?;
        Ok(perms
            .into_iter()
            .map(|p| RolePermissionDto {
                id: p.id,
                role_id: p.role_id,
                module_group_id: p.module_group_id,
                module_id: p.module_id,
                can_create: p.can_create,
                can_read: p.can_read,
                can_update: p.can_update,
                can_delete: p.can_delete,
                created_at: p.created_at,
                modify_at: p.modify_at,
            })
            .collect())
    }

    async fn create_permission(
        &self,
        req: CreateRolePermissionRequest,
    ) -> Result<RolePermissionDto, AppError> {
        req.validate()?;

        let group_id = match req.module_group_id {
            Some(g) => g,
            None => self
                .repo
                .find_module_group_id(&req.module_id)
                .await?
                .ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "Module with ID '{}' does not exist or has no group",
                        req.module_id
                    ))
                })?,
        };

        let now = Utc::now();
        let perm_id = format!("perm-{}", Uuid::new_v4());

        let can_create = req.can_create.unwrap_or(false);
        let can_read = req.can_read.unwrap_or(false);
        let can_update = req.can_update.unwrap_or(false);
        let can_delete = req.can_delete.unwrap_or(false);

        let (can_create, can_read, can_update, can_delete) =
            normalize_permissions(can_create, can_read, can_update, can_delete);

        let entity = RolePermissionEntity {
            id: perm_id,
            role_id: req.role_id,
            module_group_id: group_id,
            module_id: req.module_id,
            can_create,
            can_read,
            can_update,
            can_delete,
            created_at: now,
            created_by: Some("system".to_string()),
            modify_at: now,
            modify_by: Some("system".to_string()),
        };

        let created = self.repo.create(&entity).await?;

        Ok(RolePermissionDto {
            id: created.id,
            role_id: created.role_id,
            module_group_id: created.module_group_id,
            module_id: created.module_id,
            can_create: created.can_create,
            can_read: created.can_read,
            can_update: created.can_update,
            can_delete: created.can_delete,
            created_at: created.created_at,
            modify_at: created.modify_at,
        })
    }

    async fn update_permission(
        &self,
        id: &str,
        req: UpdateRolePermissionRequest,
    ) -> Result<RolePermissionDto, AppError> {
        req.validate()?;

        let mut perm =
            self.repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Permission with ID '{id}' not found"))
            })?;

        if let Some(c) = req.can_create {
            perm.can_create = c;
        }
        if let Some(r) = req.can_read {
            perm.can_read = r;
        }
        if let Some(u) = req.can_update {
            perm.can_update = u;
        }
        if let Some(d) = req.can_delete {
            perm.can_delete = d;
        }

        let (c, r, u, d) = normalize_permissions(
            perm.can_create,
            perm.can_read,
            perm.can_update,
            perm.can_delete,
        );
        perm.can_create = c;
        perm.can_read = r;
        perm.can_update = u;
        perm.can_delete = d;

        perm.modify_at = Utc::now();
        perm.modify_by = Some("system".to_string());

        let updated = self.repo.update(&perm).await?;

        Ok(RolePermissionDto {
            id: updated.id,
            role_id: updated.role_id,
            module_group_id: updated.module_group_id,
            module_id: updated.module_id,
            can_create: updated.can_create,
            can_read: updated.can_read,
            can_update: updated.can_update,
            can_delete: updated.can_delete,
            created_at: updated.created_at,
            modify_at: updated.modify_at,
        })
    }

    async fn delete_permission(&self, id: &str) -> Result<(), AppError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_permissions_formula() {
        // Simulasi 1 Kasus A dari PRD 3.4: Mutasi aktif tanpa read -> auto-enable read
        let (c, r, u, d) = normalize_permissions(true, false, false, false);
        assert!(c);
        assert!(r);
        assert!(!u);
        assert!(!d);

        // Simulasi 1 Kasus B dari PRD 3.4: Semua nonaktif -> tetap false
        let (c, r, u, d) = normalize_permissions(false, false, false, false);
        assert!(!c);
        assert!(!r);
        assert!(!u);
        assert!(!d);

        // Kasus Update/Delete aktif -> auto-enable read
        let (_, r, _, _) = normalize_permissions(false, false, true, false);
        assert!(r);
        let (_, r, _, _) = normalize_permissions(false, false, false, true);
        assert!(r);
    }

    #[test]
    fn test_access_evaluation_formula() {
        // Rumus dari PRD 3.2: AksesDiberikan = RoleActive && GroupActive && ModuleActive && can_read
        assert!(is_access_granted(true, true, true, true));
        assert!(!is_access_granted(false, true, true, true));
        assert!(!is_access_granted(true, false, true, true));
        assert!(!is_access_granted(true, true, false, true));
        assert!(!is_access_granted(true, true, true, false));
    }

    #[tokio::test]
    async fn test_simulation_3_get_role_by_id_tree() {
        struct MockRoleRepo;
        #[async_trait]
        impl RoleRepository for MockRoleRepo {
            async fn find_all(&self) -> Result<Vec<RoleEntity>, AppError> {
                Ok(vec![])
            }
            async fn find_by_id(&self, id: &str) -> Result<Option<RoleEntity>, AppError> {
                if id == "role-uuid-1" {
                    Ok(Some(RoleEntity {
                        id: "role-uuid-1".to_string(),
                        name: "ADMINISTRATOR".to_string(),
                        is_active: true,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            async fn find_by_name(&self, _name: &str) -> Result<Option<RoleEntity>, AppError> {
                Ok(None)
            }
            async fn create(&self, _role: &RoleEntity) -> Result<RoleEntity, AppError> {
                unreachable!()
            }
            async fn update(&self, _role: &RoleEntity) -> Result<RoleEntity, AppError> {
                unreachable!()
            }
            async fn delete(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
            async fn count_users_by_role_id(&self, _role_id: &str) -> Result<i64, AppError> {
                Ok(0)
            }
            async fn find_groups_and_modules_for_role(
                &self,
                _role_id: &str,
            ) -> Result<
                (
                    Vec<crate::modules::system_modules::dto::GroupEntity>,
                    Vec<ModuleEntity>,
                    Vec<RolePermissionEntity>,
                ),
                AppError,
            > {
                let groups = vec![crate::modules::system_modules::dto::GroupEntity {
                    id: "group-uuid-1".to_string(),
                    name: "Master Data".to_string(),
                    icon: Some("LuDatabase".to_string()),
                    is_active: true,
                    created_at: Utc::now(),
                    created_by: None,
                    modify_at: Utc::now(),
                    modify_by: None,
                }];

                let modules = vec![ModuleEntity {
                    id: "mod-uuid-1".to_string(),
                    group_id: "group-uuid-1".to_string(),
                    parent_id: None,
                    name: "Tanaman".to_string(),
                    path: "/master/plants".to_string(),
                    icon: Some("LuSprout".to_string()),
                    is_active: true,
                    created_at: Utc::now(),
                    created_by: None,
                    modify_at: Utc::now(),
                    modify_by: None,
                }];

                let permissions = vec![RolePermissionEntity {
                    id: "perm-1".to_string(),
                    role_id: "role-uuid-1".to_string(),
                    module_group_id: "group-uuid-1".to_string(),
                    module_id: "mod-uuid-1".to_string(),
                    can_create: true,
                    can_read: true,
                    can_update: true,
                    can_delete: false,
                    created_at: Utc::now(),
                    created_by: None,
                    modify_at: Utc::now(),
                    modify_by: None,
                }];

                Ok((groups, modules, permissions))
            }
        }

        let repo = Arc::new(MockRoleRepo);
        let service = RoleServiceImpl::new(repo);
        let detail = service
            .get_role_detail("role-uuid-1")
            .await
            .expect("Detail should succeed");

        assert_eq!(detail.id, "role-uuid-1");
        assert_eq!(detail.name, "ADMINISTRATOR");
        assert_eq!(detail.module_group.len(), 1);
        assert_eq!(detail.module_group[0].name, "Master Data");
        assert_eq!(detail.module_group[0].module.len(), 1);
        assert_eq!(detail.module_group[0].module[0].name, "Tanaman");
        assert_eq!(detail.module_group[0].module[0].role_permission.len(), 1);
        assert!(detail.module_group[0].module[0].role_permission[0].can_read);
        assert!(detail.module_group[0].module[0].role_permission[0].can_create);
        assert!(detail.module_group[0].module[0].role_permission[0].can_update);
        assert!(!detail.module_group[0].module[0].role_permission[0].can_delete);
    }

    #[tokio::test]
    async fn test_simulation_role_detail_filters_modules_without_access() {
        struct MockFilteredRoleRepo;
        #[async_trait]
        impl RoleRepository for MockFilteredRoleRepo {
            async fn find_all(&self) -> Result<Vec<RoleEntity>, AppError> {
                Ok(vec![])
            }
            async fn find_by_id(&self, id: &str) -> Result<Option<RoleEntity>, AppError> {
                if id == "role-1" {
                    Ok(Some(RoleEntity {
                        id: "role-1".to_string(),
                        name: "OPERATOR".to_string(),
                        is_active: true,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            async fn find_by_name(&self, _name: &str) -> Result<Option<RoleEntity>, AppError> {
                Ok(None)
            }
            async fn create(&self, _role: &RoleEntity) -> Result<RoleEntity, AppError> {
                unreachable!()
            }
            async fn update(&self, _role: &RoleEntity) -> Result<RoleEntity, AppError> {
                unreachable!()
            }
            async fn delete(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
            async fn count_users_by_role_id(&self, _role_id: &str) -> Result<i64, AppError> {
                Ok(0)
            }
            async fn find_groups_and_modules_for_role(
                &self,
                _role_id: &str,
            ) -> Result<
                (
                    Vec<crate::modules::system_modules::dto::GroupEntity>,
                    Vec<ModuleEntity>,
                    Vec<RolePermissionEntity>,
                ),
                AppError,
            > {
                let groups = vec![crate::modules::system_modules::dto::GroupEntity {
                    id: "grp-1".to_string(),
                    name: "Operations".to_string(),
                    icon: None,
                    is_active: true,
                    created_at: Utc::now(),
                    created_by: None,
                    modify_at: Utc::now(),
                    modify_by: None,
                }];

                // mod-allowed has can_read = true, mod-forbidden has can_read = false
                let modules = vec![
                    ModuleEntity {
                        id: "mod-allowed".to_string(),
                        group_id: "grp-1".to_string(),
                        parent_id: None,
                        name: "Harvest List".to_string(),
                        path: "/harvest".to_string(),
                        icon: None,
                        is_active: true,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    },
                    ModuleEntity {
                        id: "mod-forbidden".to_string(),
                        group_id: "grp-1".to_string(),
                        parent_id: None,
                        name: "Secret Settings".to_string(),
                        path: "/settings/secret".to_string(),
                        icon: None,
                        is_active: true,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    },
                ];

                let permissions = vec![
                    RolePermissionEntity {
                        id: "perm-allowed".to_string(),
                        role_id: "role-1".to_string(),
                        module_group_id: "grp-1".to_string(),
                        module_id: "mod-allowed".to_string(),
                        can_create: false,
                        can_read: true,
                        can_update: false,
                        can_delete: false,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    },
                    RolePermissionEntity {
                        id: "perm-forbidden".to_string(),
                        role_id: "role-1".to_string(),
                        module_group_id: "grp-1".to_string(),
                        module_id: "mod-forbidden".to_string(),
                        can_create: false,
                        can_read: false, // Access not granted!
                        can_update: false,
                        can_delete: false,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    },
                ];

                Ok((groups, modules, permissions))
            }
        }

        let repo = Arc::new(MockFilteredRoleRepo);
        let service = RoleServiceImpl::new(repo);
        let detail = service
            .get_role_detail("role-1")
            .await
            .expect("Detail should succeed");

        assert_eq!(detail.module_group.len(), 1);
        let modules = &detail.module_group[0].module;
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].id, "mod-allowed");
        assert_eq!(modules[0].name, "Harvest List");
    }
}
