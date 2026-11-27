use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::common::error::AppError;
use crate::middleware::auth::Claims;
use crate::modules::auth::dto::{
    LoginRequest, LoginResponse, LoginUserResponse, RegisterRequest, RegisterResponse, UserEntity,
};
use crate::modules::auth::repository::AuthRepository;

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register(&self, req: RegisterRequest) -> Result<RegisterResponse, AppError>;
    async fn login(&self, req: LoginRequest) -> Result<LoginResponse, AppError>;
}

pub struct AuthServiceImpl {
    repo: Arc<dyn AuthRepository>,
    jwt_secret: String,
    jwt_expiration_hours: i64,
}

impl AuthServiceImpl {
    pub fn new(
        repo: Arc<dyn AuthRepository>,
        jwt_secret: String,
        jwt_expiration_hours: i64,
    ) -> Self {
        Self {
            repo,
            jwt_secret,
            jwt_expiration_hours,
        }
    }

    pub fn hash_password(password: &str) -> Result<String, AppError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        bcrypt::verify(password, hash)
            .map_err(|e| AppError::Internal(format!("Failed to verify password: {e}")))
    }

    pub fn generate_token(
        &self,
        user_id: &str,
        username: &str,
        role_id: &str,
    ) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(self.jwt_expiration_hours))
            .ok_or_else(|| AppError::Internal("Timestamp overflow".to_string()))?
            .timestamp();

        let claims = Claims {
            user_id: user_id.to_string(),
            username: username.to_string(),
            role_id: role_id.to_string(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate JWT: {e}")))
    }
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(&self, mut req: RegisterRequest) -> Result<RegisterResponse, AppError> {
        req.validate()?;
        req.username = crate::common::validation::validate_username(&req.username)?;

        if self
            .repo
            .find_by_username_or_email(&req.username, &req.email)
            .await?
            .is_some()
        {
            return Err(AppError::BadRequest(
                "Username or email already exists".to_string(),
            ));
        }

        if !self.repo.role_exists(&req.role_id).await? {
            return Err(AppError::BadRequest(format!(
                "Role with ID '{}' does not exist",
                req.role_id
            )));
        }

        let hashed_password = Self::hash_password(&req.password)?;
        let now = Utc::now();
        let user_id = format!("usr_{}", Uuid::new_v4());

        let new_user = UserEntity {
            id: user_id,
            name: req.name,
            email: req.email,
            username: req.username,
            password: hashed_password,
            role_id: req.role_id,
            is_active: true,
            created_at: now,
            created_by: Some("system".to_string()),
            modify_at: now,
            modify_by: Some("system".to_string()),
        };

        let created = self.repo.create_user(&new_user).await?;

        Ok(RegisterResponse {
            id: created.id,
            name: created.name,
            email: created.email,
            username: created.username,
            role_id: created.role_id,
            is_active: created.is_active,
            created_at: created.created_at,
        })
    }

    async fn login(&self, req: LoginRequest) -> Result<LoginResponse, AppError> {
        req.validate()?;

        let user = self
            .repo
            .find_by_username(&req.username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

        if !user.is_active {
            return Err(AppError::Unauthorized(
                "User account is inactive".to_string(),
            ));
        }

        let is_password_valid = Self::verify_password(&req.password, &user.password)?;
        if !is_password_valid {
            return Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        let token = self.generate_token(&user.id, &user.username, &user.role_id)?;

        Ok(LoginResponse {
            token,
            user: LoginUserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                username: user.username,
                role_id: user.role_id,
                is_active: user.is_active,
                created_at: user.created_at,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockAuthRepo {
        users: Mutex<Vec<UserEntity>>,
        roles: Vec<String>,
    }

    #[async_trait]
    impl AuthRepository for MockAuthRepo {
        async fn find_by_username(&self, username: &str) -> Result<Option<UserEntity>, AppError> {
            let users = self
                .users
                .lock()
                .map_err(|_| AppError::Internal("Mutex error".to_string()))?;
            Ok(users.iter().find(|u| u.username == username).cloned())
        }

        async fn find_by_username_or_email(
            &self,
            username: &str,
            email: &str,
        ) -> Result<Option<UserEntity>, AppError> {
            let users = self
                .users
                .lock()
                .map_err(|_| AppError::Internal("Mutex error".to_string()))?;
            Ok(users
                .iter()
                .find(|u| u.username == username || u.email == email)
                .cloned())
        }

        async fn role_exists(&self, role_id: &str) -> Result<bool, AppError> {
            Ok(self.roles.contains(&role_id.to_string()))
        }

        async fn create_user(&self, user: &UserEntity) -> Result<UserEntity, AppError> {
            let mut users = self
                .users
                .lock()
                .map_err(|_| AppError::Internal("Mutex error".to_string()))?;
            users.push(user.clone());
            Ok(user.clone())
        }
    }

    #[tokio::test]
    async fn test_simulation_1_register_and_login_success() {
        let repo = Arc::new(MockAuthRepo {
            users: Mutex::new(Vec::new()),
            roles: vec!["role-uuid-1".to_string()],
        });

        let service = AuthServiceImpl::new(repo, "test_jwt_secret_key_1234567890".to_string(), 24);

        // Simulasi 1 Data: Register
        let reg_req = RegisterRequest {
            name: "Budi Santoso".to_string(),
            username: "budi".to_string(),
            email: "budi@agritech.com".to_string(),
            password: "SecretPassword123".to_string(),
            role_id: "role-uuid-1".to_string(),
        };

        let reg_res = service
            .register(reg_req)
            .await
            .expect("Registration should succeed");
        assert_eq!(reg_res.username, "budi");
        assert_eq!(reg_res.email, "budi@agritech.com");
        assert_eq!(reg_res.role_id, "role-uuid-1");
        assert!(reg_res.is_active);

        // Simulasi 1 Data: Login Success
        let login_req = LoginRequest {
            username: "budi".to_string(),
            password: "SecretPassword123".to_string(),
        };

        let login_res = service
            .login(login_req)
            .await
            .expect("Login should succeed");
        assert!(!login_res.token.is_empty());
        assert_eq!(login_res.user.username, "budi");
        assert_eq!(login_res.user.email, "budi@agritech.com");
    }

    #[tokio::test]
    async fn test_simulation_2_wrong_password_unauthorized() {
        let repo = Arc::new(MockAuthRepo {
            users: Mutex::new(Vec::new()),
            roles: vec!["role-uuid-1".to_string()],
        });

        let service = AuthServiceImpl::new(repo, "test_jwt_secret_key_1234567890".to_string(), 24);

        let reg_req = RegisterRequest {
            name: "Budi Santoso".to_string(),
            username: "budi".to_string(),
            email: "budi@agritech.com".to_string(),
            password: "SecretPassword123".to_string(),
            role_id: "role-uuid-1".to_string(),
        };
        service
            .register(reg_req)
            .await
            .expect("Registration should succeed");

        // Simulasi 2: Wrong Password
        let login_req = LoginRequest {
            username: "budi".to_string(),
            password: "WrongPassword".to_string(),
        };

        let login_res = service.login(login_req).await;
        assert!(matches!(login_res, Err(AppError::Unauthorized(_))));
    }

    #[test]
    fn test_bcrypt_formula_cost_factor() {
        let password = "SecretPassword123";
        let hash = AuthServiceImpl::hash_password(password).expect("Hash should succeed");
        // Bcrypt prefix check ($2a$, $2b$, or $2y$)
        assert!(hash.starts_with("$2b$") || hash.starts_with("$2a$") || hash.starts_with("$2y$"));
        let valid =
            AuthServiceImpl::verify_password(password, &hash).expect("Verification should succeed");
        assert!(valid);
    }
}
