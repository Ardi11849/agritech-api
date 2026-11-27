use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::modules::auth::dto::UserEntity;
use crate::modules::user::dto::{
    CreateUserRequest, UpdateUserRequest, UserItemResponse, UserProfileResponse,
};
use crate::modules::user::repository::UserRepository;

pub fn resolve_updated_password_hash(
    existing_hash: &str,
    new_password: Option<&str>,
) -> Result<String, AppError> {
    match new_password {
        None => Ok(existing_hash.to_string()),
        Some(p) if p.trim().is_empty() => Ok(existing_hash.to_string()),
        Some(p) => {
            let trimmed = p.trim();
            if trimmed.len() < 6 {
                return Err(AppError::BadRequest(
                    "Password must be at least 6 characters".to_string(),
                ));
            }
            bcrypt::hash(trimmed, 12)
                .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))
        }
    }
}

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_profile(&self, user_id: &str) -> Result<UserProfileResponse, AppError>;
    async fn get_users(&self) -> Result<Vec<UserItemResponse>, AppError>;
    async fn get_user_by_id(&self, id: &str) -> Result<UserItemResponse, AppError>;
    async fn create_user(
        &self,
        req: CreateUserRequest,
        actor: &str,
    ) -> Result<UserItemResponse, AppError>;
    async fn update_user(
        &self,
        id: &str,
        req: UpdateUserRequest,
        actor: &str,
    ) -> Result<UserItemResponse, AppError>;
    async fn delete_user(&self, target_id: &str, actor_id: &str) -> Result<(), AppError>;
}

pub struct UserServiceImpl {
    repo: Arc<dyn UserRepository>,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_profile(&self, user_id: &str) -> Result<UserProfileResponse, AppError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID '{user_id}' not found")))?;

        Ok(UserProfileResponse {
            user_id: user.id,
            username: user.username,
            role_id: user.role_id,
        })
    }

    async fn get_users(&self) -> Result<Vec<UserItemResponse>, AppError> {
        self.repo.find_all_users().await
    }

    async fn get_user_by_id(&self, id: &str) -> Result<UserItemResponse, AppError> {
        self.repo
            .find_user_item_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID '{id}' not found")))
    }

    async fn create_user(
        &self,
        mut req: CreateUserRequest,
        actor: &str,
    ) -> Result<UserItemResponse, AppError> {
        req.validate()?;
        req.username = crate::common::validation::validate_username(&req.username)?;

        if !self.repo.role_exists(&req.role_id).await? {
            return Err(AppError::BadRequest(format!(
                "Role with ID '{}' not found",
                req.role_id
            )));
        }

        if self
            .repo
            .find_by_username_or_email_excluding(&req.username, &req.email, None)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(
                "Username or email is already taken by another account".to_string(),
            ));
        }

        let password_hash = bcrypt::hash(req.password.trim(), 12)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?;

        let now = Utc::now();
        let user_id = format!("usr_{}", Uuid::new_v4());

        let entity = UserEntity {
            id: user_id.clone(),
            name: req.name.clone(),
            email: req.email.clone(),
            username: req.username.clone(),
            password: password_hash,
            role_id: req.role_id.clone(),
            is_active: req.is_active.unwrap_or(true),
            created_at: now,
            created_by: Some(actor.to_string()),
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let created = self.repo.create_user(&entity).await?;
        let role_name = self
            .repo
            .get_role_name(&created.role_id)
            .await?
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Ok(UserItemResponse {
            id: created.id,
            name: created.name,
            email: created.email,
            username: created.username,
            role_id: created.role_id,
            role_name,
            is_active: created.is_active,
            created_at: created.created_at,
            created_by: created.created_by,
            modify_at: created.modify_at,
            modify_by: created.modify_by,
        })
    }

    async fn update_user(
        &self,
        id: &str,
        mut req: UpdateUserRequest,
        actor: &str,
    ) -> Result<UserItemResponse, AppError> {
        req.validate()?;
        req.username = crate::common::validation::validate_username(&req.username)?;

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID '{id}' not found")))?;

        let is_protected_superadmin = existing.username.trim().eq_ignore_ascii_case("admin")
            || existing.username.trim().eq_ignore_ascii_case("ardi");
        if is_protected_superadmin && req.is_active == Some(false) {
            return Err(AppError::BadRequest(
                "Superadmin account cannot be deactivated".to_string(),
            ));
        }

        if !self.repo.role_exists(&req.role_id).await? {
            return Err(AppError::BadRequest(format!(
                "Role with ID '{}' not found",
                req.role_id
            )));
        }

        if self
            .repo
            .find_by_username_or_email_excluding(&req.username, &req.email, Some(id))
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(
                "Username or email is already taken by another account".to_string(),
            ));
        }

        let password_hash =
            resolve_updated_password_hash(&existing.password, req.password.as_deref())?;

        let now = Utc::now();
        let entity = UserEntity {
            id: id.to_string(),
            name: req.name,
            email: req.email,
            username: req.username,
            password: password_hash,
            role_id: req.role_id,
            is_active: req.is_active.unwrap_or(existing.is_active),
            created_at: existing.created_at,
            created_by: existing.created_by,
            modify_at: now,
            modify_by: Some(actor.to_string()),
        };

        let updated = self.repo.update_user(&entity).await?;
        let role_name = self
            .repo
            .get_role_name(&updated.role_id)
            .await?
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Ok(UserItemResponse {
            id: updated.id,
            name: updated.name,
            email: updated.email,
            username: updated.username,
            role_id: updated.role_id,
            role_name,
            is_active: updated.is_active,
            created_at: updated.created_at,
            created_by: updated.created_by,
            modify_at: updated.modify_at,
            modify_by: updated.modify_by,
        })
    }

    async fn delete_user(&self, target_id: &str, actor_id: &str) -> Result<(), AppError> {
        if target_id == actor_id {
            return Err(AppError::BadRequest(
                "Cannot delete your own active account".to_string(),
            ));
        }

        let target_user =
            self.repo.find_by_id(target_id).await?.ok_or_else(|| {
                AppError::NotFound(format!("User with ID '{target_id}' not found"))
            })?;

        let is_protected_superadmin = target_user.username.trim().eq_ignore_ascii_case("admin")
            || target_user.username.trim().eq_ignore_ascii_case("ardi");
        if is_protected_superadmin {
            return Err(AppError::BadRequest(
                "Superadmin account is protected and cannot be deleted".to_string(),
            ));
        }

        self.repo.delete_user(target_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conditional_password_update_formula() {
        let old_hash = "$2b$12$oldhashedpasswordvalue1234567890";

        // Kasus 1: None -> keep old hash
        assert_eq!(
            resolve_updated_password_hash(old_hash, None).unwrap(),
            old_hash
        );

        // Kasus 2: Empty string or spaces -> keep old hash
        assert_eq!(
            resolve_updated_password_hash(old_hash, Some("")).unwrap(),
            old_hash
        );
        assert_eq!(
            resolve_updated_password_hash(old_hash, Some("   ")).unwrap(),
            old_hash
        );

        // Kasus 3: Password too short (< 6) -> Bad Request
        assert!(matches!(
            resolve_updated_password_hash(old_hash, Some("12345")),
            Err(AppError::BadRequest(_))
        ));

        // Kasus 4: Password valid (>= 6) -> valid new hash
        let new_hash = resolve_updated_password_hash(old_hash, Some("NewSecretPassword123!"))
            .expect("Hashing valid password should succeed");
        assert_ne!(new_hash, old_hash);
        assert!(bcrypt::verify("NewSecretPassword123!", &new_hash).unwrap());
    }

    #[tokio::test]
    async fn test_self_deletion_and_superadmin_guard() {
        struct MockUserRepo;
        #[async_trait]
        impl UserRepository for MockUserRepo {
            async fn find_by_id(&self, id: &str) -> Result<Option<UserEntity>, AppError> {
                if id == "admin-user" {
                    Ok(Some(UserEntity {
                        id: "admin-user".to_string(),
                        name: "Admin".to_string(),
                        email: "admin@test.com".to_string(),
                        username: "admin".to_string(),
                        password: "hash".to_string(),
                        role_id: "role-1".to_string(),
                        is_active: true,
                        created_at: Utc::now(),
                        created_by: None,
                        modify_at: Utc::now(),
                        modify_by: None,
                    }))
                } else if id == "user-1" {
                    Ok(Some(UserEntity {
                        id: "user-1".to_string(),
                        name: "User 1".to_string(),
                        email: "user1@test.com".to_string(),
                        username: "user1".to_string(),
                        password: "hash".to_string(),
                        role_id: "role-1".to_string(),
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
            async fn find_all_users(&self) -> Result<Vec<UserItemResponse>, AppError> {
                Ok(vec![])
            }
            async fn find_user_item_by_id(
                &self,
                _id: &str,
            ) -> Result<Option<UserItemResponse>, AppError> {
                Ok(None)
            }
            async fn find_by_username_or_email_excluding(
                &self,
                _u: &str,
                _e: &str,
                _ex: Option<&str>,
            ) -> Result<Option<UserEntity>, AppError> {
                Ok(None)
            }
            async fn create_user(&self, _u: &UserEntity) -> Result<UserEntity, AppError> {
                unreachable!()
            }
            async fn update_user(&self, _u: &UserEntity) -> Result<UserEntity, AppError> {
                unreachable!()
            }
            async fn delete_user(&self, _id: &str) -> Result<(), AppError> {
                Ok(())
            }
            async fn role_exists(&self, _role_id: &str) -> Result<bool, AppError> {
                Ok(true)
            }
            async fn get_role_name(&self, _role_id: &str) -> Result<Option<String>, AppError> {
                Ok(Some("ADMINISTRATOR".to_string()))
            }
        }

        let repo = Arc::new(MockUserRepo);
        let service = UserServiceImpl::new(repo);

        // Self deletion must be rejected
        let res_self = service.delete_user("user-1", "user-1").await;
        assert!(
            matches!(res_self, Err(AppError::BadRequest(msg)) if msg.contains("Cannot delete your own"))
        );

        // Superadmin deletion must be rejected even by another admin
        let res_superadmin = service.delete_user("admin-user", "admin-2").await;
        assert!(
            matches!(res_superadmin, Err(AppError::BadRequest(msg)) if msg.contains("Superadmin account is protected"))
        );
    }
}
