pub mod common;
pub mod config;
pub mod docs;
pub mod infra;
pub mod middleware;
pub mod modules;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::common::response::ApiResponse;
use crate::config::AppConfig;
use crate::modules::auth::repository::PgAuthRepository;
use crate::modules::auth::service::{AuthService, AuthServiceImpl};
use crate::modules::daily_readings::repository::PgDailyReadingRepository;
use crate::modules::daily_readings::service::{DailyReadingService, DailyReadingServiceImpl};
use crate::modules::finance::hpp::repository::PgHppRepository;
use crate::modules::finance::hpp::service::{HppService, HppServiceImpl};
use crate::modules::finance::purchases::repository::PgPurchaseRepository;
use crate::modules::finance::purchases::service::{PurchaseService, PurchaseServiceImpl};
use crate::modules::finance::sales::repository::PgSaleRepository;
use crate::modules::finance::sales::service::{SaleService, SaleServiceImpl};
use crate::modules::finance::tax::repository::PgTaxRepository;
use crate::modules::finance::tax::service::{TaxService, TaxServiceImpl};
use crate::modules::health::repository::PgHealthRepository;
use crate::modules::health::service::{HealthService, HealthServiceImpl};
use crate::modules::inventory::repository::PgInventoryRepository;
use crate::modules::inventory::service::{InventoryService, InventoryServiceImpl};
use crate::modules::masters::customers::repository::PgCustomerRepository;
use crate::modules::masters::customers::service::{CustomerService, CustomerServiceImpl};
use crate::modules::masters::installations::repository::PgInstallationRepository;
use crate::modules::masters::installations::service::{
    InstallationService, InstallationServiceImpl,
};
use crate::modules::masters::item_categories::repository::PgItemCategoryRepository;
use crate::modules::masters::item_categories::service::{
    ItemCategoryService, ItemCategoryServiceImpl,
};
use crate::modules::masters::items::repository::PgItemRepository;
use crate::modules::masters::items::service::{ItemService, ItemServiceImpl};
use crate::modules::masters::locations::repository::PgLocationRepository;
use crate::modules::masters::locations::service::{LocationService, LocationServiceImpl};
use crate::modules::masters::plants::repository::PgPlantRepository;
use crate::modules::masters::plants::service::{PlantService, PlantServiceImpl};
use crate::modules::masters::suppliers::repository::PgSupplierRepository;
use crate::modules::masters::suppliers::service::{SupplierService, SupplierServiceImpl};
use crate::modules::plan::sop::repository::PgSopRepository;
use crate::modules::plan::sop::service::{SopService, SopServiceImpl};
use crate::modules::plan::tasks::repository::PgTaskRepository;
use crate::modules::plan::tasks::service::{TaskService, TaskServiceImpl};
use crate::modules::role::repository::{PgRolePermissionRepository, PgRoleRepository};
use crate::modules::role::service::{
    RolePermissionService, RolePermissionServiceImpl, RoleService, RoleServiceImpl,
};
use crate::modules::sessions::repository::PgSessionRepository;
use crate::modules::sessions::service::{SessionService, SessionServiceImpl};
use crate::modules::system_modules::repository::PgSystemModulesRepository;
use crate::modules::system_modules::service::{SystemModulesService, SystemModulesServiceImpl};
use crate::modules::user::repository::PgUserRepository;
use crate::modules::user::service::{UserService, UserServiceImpl};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<AppConfig>,
    pub health_service: Arc<dyn HealthService>,
    pub auth_service: Arc<dyn AuthService>,
    pub user_service: Arc<dyn UserService>,
    pub role_service: Arc<dyn RoleService>,
    pub role_permission_service: Arc<dyn RolePermissionService>,
    pub system_modules_service: Arc<dyn SystemModulesService>,
    pub plant_service: Arc<dyn PlantService>,
    pub location_service: Arc<dyn LocationService>,
    pub installation_service: Arc<dyn InstallationService>,
    pub item_category_service: Arc<dyn ItemCategoryService>,
    pub item_service: Arc<dyn ItemService>,
    pub supplier_service: Arc<dyn SupplierService>,
    pub customer_service: Arc<dyn CustomerService>,
    pub inventory_service: Arc<dyn InventoryService>,
    pub session_service: Arc<dyn SessionService>,
    pub daily_reading_service: Arc<dyn DailyReadingService>,
    pub sop_service: Arc<dyn SopService>,
    pub task_service: Arc<dyn TaskService>,
    pub purchase_service: Arc<dyn PurchaseService>,
    pub sale_service: Arc<dyn SaleService>,
    pub hpp_service: Arc<dyn HppService>,
    pub tax_service: Arc<dyn TaxService>,
}

pub async fn fallback_handler() -> impl IntoResponse {
    ApiResponse::error_response(
        StatusCode::NOT_FOUND,
        "Not Found",
        "The requested resource was not found",
    )
}

pub fn create_app(state: AppState) -> Router {
    let mut app = modules::api_router()
        .fallback(fallback_handler)
        .layer(CorsLayer::permissive());

    if state.config.enable_swagger {
        app = docs::mount_swagger_ui(app);
    }

    app.with_state(state)
}

pub fn build_app_state(pool: PgPool, config: AppConfig) -> AppState {
    let config_arc = Arc::new(config);

    let health_repo = Arc::new(PgHealthRepository::new(pool.clone()));
    let health_service = Arc::new(HealthServiceImpl::new(health_repo));

    let auth_repo = Arc::new(PgAuthRepository::new(pool.clone()));
    let auth_service = Arc::new(AuthServiceImpl::new(
        auth_repo,
        config_arc.jwt_secret.clone(),
        config_arc.jwt_expiration_hours,
    ));

    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let user_service = Arc::new(UserServiceImpl::new(user_repo));

    let role_repo = Arc::new(PgRoleRepository::new(pool.clone()));
    let role_service = Arc::new(RoleServiceImpl::new(role_repo));

    let role_perm_repo = Arc::new(PgRolePermissionRepository::new(pool.clone()));
    let role_permission_service = Arc::new(RolePermissionServiceImpl::new(role_perm_repo));

    let system_modules_repo = Arc::new(PgSystemModulesRepository::new(pool.clone()));
    let system_modules_service = Arc::new(SystemModulesServiceImpl::new(system_modules_repo));

    let plant_repo = Arc::new(PgPlantRepository::new(pool.clone()));
    let plant_service = Arc::new(PlantServiceImpl::new(plant_repo.clone()));

    let location_repo = Arc::new(PgLocationRepository::new(pool.clone()));
    let location_service = Arc::new(LocationServiceImpl::new(location_repo.clone()));

    let installation_repo = Arc::new(PgInstallationRepository::new(pool.clone()));
    let installation_service = Arc::new(InstallationServiceImpl::new(installation_repo.clone()));

    let item_cat_repo = Arc::new(PgItemCategoryRepository::new(pool.clone()));
    let item_category_service = Arc::new(ItemCategoryServiceImpl::new(item_cat_repo));

    let item_repo = Arc::new(PgItemRepository::new(pool.clone()));
    let item_service = Arc::new(ItemServiceImpl::new(item_repo));

    let supplier_repo = Arc::new(PgSupplierRepository::new(pool.clone()));
    let supplier_service = Arc::new(SupplierServiceImpl::new(supplier_repo));

    let customer_repo = Arc::new(PgCustomerRepository::new(pool.clone()));
    let customer_service = Arc::new(CustomerServiceImpl::new(customer_repo));

    let inventory_repo = Arc::new(PgInventoryRepository::new(pool.clone()));
    let inventory_service = Arc::new(InventoryServiceImpl::new(inventory_repo));

    let session_repo = Arc::new(PgSessionRepository::new(pool.clone()));
    let session_service = Arc::new(SessionServiceImpl::new(
        session_repo.clone(),
        plant_repo,
        location_repo,
        installation_repo,
    ));

    let daily_reading_repo = Arc::new(PgDailyReadingRepository::new(pool.clone()));
    let daily_reading_service = Arc::new(DailyReadingServiceImpl::new(
        daily_reading_repo,
        session_repo,
    ));

    let sop_repo = Arc::new(PgSopRepository::new(pool.clone()));
    let sop_service = Arc::new(SopServiceImpl::new(sop_repo));

    let task_repo = Arc::new(PgTaskRepository::new(pool.clone()));
    let task_service = Arc::new(TaskServiceImpl::new(task_repo));

    let purchase_repo = Arc::new(PgPurchaseRepository::new(pool.clone()));
    let purchase_service = Arc::new(PurchaseServiceImpl::new(purchase_repo));

    let sale_repo = Arc::new(PgSaleRepository::new(pool.clone()));
    let sale_service = Arc::new(SaleServiceImpl::new(sale_repo));

    let hpp_repo = Arc::new(PgHppRepository::new(pool.clone()));
    let hpp_service = Arc::new(HppServiceImpl::new(hpp_repo));

    let tax_repo = Arc::new(PgTaxRepository::new(pool.clone()));
    let tax_service = Arc::new(TaxServiceImpl::new(tax_repo));

    AppState {
        db: pool,
        config: config_arc,
        health_service,
        auth_service,
        user_service,
        role_service,
        role_permission_service,
        system_modules_service,
        plant_service,
        location_service,
        installation_service,
        item_category_service,
        item_service,
        supplier_service,
        customer_service,
        inventory_service,
        session_service,
        daily_reading_service,
        sop_service,
        task_service,
        purchase_service,
        sale_service,
        hpp_service,
        tax_service,
    }
}
