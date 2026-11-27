use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use std::str::FromStr;
use std::sync::Arc;
use tower::ServiceExt;

use agritech_api::common::error::AppError;
use agritech_api::config::AppConfig;
use agritech_api::create_app;
use agritech_api::modules::auth::dto::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse,
};
use agritech_api::modules::auth::service::AuthService;
use agritech_api::modules::daily_readings::dto::{
    CreateDailyReadingRequest, DailyReadingItem, DailyReadingListQuery,
    SessionDailyReadingsResponse, UnreportedTodayQuery, UnreportedTodayResponse,
};
use agritech_api::modules::daily_readings::service::DailyReadingService;
use agritech_api::modules::health::dto::{ComponentStatus, HealthStatus};
use agritech_api::modules::health::service::HealthService;
use agritech_api::modules::inventory::dto::{
    CreateMovementRequest, CreateStockOpnameRequest, InventoryKpi, InventoryLedgerResponse,
    InventorySummaryQuery, InventorySummaryResponse, StockCardQuery, StockCardResponse,
};
use agritech_api::modules::inventory::service::InventoryService;
use agritech_api::modules::masters::customers::dto::{
    CreateCustomerRequest, CustomerListQuery, CustomerResponse, UpdateCustomerRequest,
};
use agritech_api::modules::masters::customers::service::CustomerService;
use agritech_api::modules::masters::installations::dto::{
    CreateInstallationRequest, InstallationListQuery, InstallationResponse,
    UpdateInstallationRequest,
};
use agritech_api::modules::masters::installations::service::InstallationService;
use agritech_api::modules::masters::item_categories::dto::{
    CreateItemCategoryRequest, ItemCategoryListQuery, ItemCategoryResponse,
    UpdateItemCategoryRequest,
};
use agritech_api::modules::masters::item_categories::service::ItemCategoryService;
use agritech_api::modules::masters::items::dto::{
    CreateMasterItemRequest, ItemListQuery, MasterItemResponse, UpdateMasterItemRequest,
};
use agritech_api::modules::masters::items::service::ItemService;
use agritech_api::modules::masters::locations::dto::{
    CreateLocationRequest, LocationListQuery, LocationResponse, UpdateLocationRequest,
};
use agritech_api::modules::masters::locations::service::LocationService;
use agritech_api::modules::masters::plants::dto::{
    CreatePlantRequest, PlantListQuery, PlantResponse, UpdatePlantRequest,
};
use agritech_api::modules::masters::plants::service::PlantService;
use agritech_api::modules::masters::suppliers::dto::{
    CreateSupplierRequest, SupplierListQuery, SupplierResponse, UpdateSupplierRequest,
};
use agritech_api::modules::masters::suppliers::service::SupplierService;
use agritech_api::modules::role::dto::{
    CreateRolePermissionRequest, CreateRoleRequest, RoleDetailResponse, RoleItemResponse,
    RolePermissionDto, UpdateRolePermissionRequest, UpdateRoleRequest,
};
use agritech_api::modules::role::service::{RolePermissionService, RoleService};
use agritech_api::modules::sessions::dto::{
    CompleteHarvestRequest, CreateSessionRequest, SessionDetailResponse, SessionKpi,
    SessionListQuery, SessionListResponse, UpdateSessionRequest,
};
use agritech_api::modules::sessions::service::SessionService;
use agritech_api::modules::system_modules::dto::{
    CreateGroupRequest, CreateModuleRequest, GroupResponse, ModuleTreeResponse, UpdateGroupRequest,
    UpdateModuleRequest,
};
use agritech_api::modules::system_modules::service::SystemModulesService;
use agritech_api::modules::user::dto::{
    CreateUserRequest, UpdateUserRequest, UserItemResponse, UserProfileResponse,
};
use agritech_api::modules::user::service::UserService;
use agritech_api::AppState;

// Mock Health
struct MockHealth;
#[async_trait]
impl HealthService for MockHealth {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus {
            app: ComponentStatus::Ok,
            database: ComponentStatus::Ok,
        }
    }
}

// Mock Auth
struct MockAuth;
#[async_trait]
impl AuthService for MockAuth {
    async fn register(&self, _req: RegisterRequest) -> Result<RegisterResponse, AppError> {
        unreachable!()
    }
    async fn login(&self, _req: LoginRequest) -> Result<LoginResponse, AppError> {
        unreachable!()
    }
}

// Mock User
struct MockUser;
#[async_trait]
impl UserService for MockUser {
    async fn get_users(&self) -> Result<Vec<UserItemResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_user_by_id(&self, _id: &str) -> Result<UserItemResponse, AppError> {
        unreachable!()
    }
    async fn create_user(
        &self,
        _req: CreateUserRequest,
        _actor: &str,
    ) -> Result<UserItemResponse, AppError> {
        unreachable!()
    }
    async fn update_user(
        &self,
        _id: &str,
        _req: UpdateUserRequest,
        _actor: &str,
    ) -> Result<UserItemResponse, AppError> {
        unreachable!()
    }
    async fn delete_user(&self, _id: &str, _current_user_id: &str) -> Result<(), AppError> {
        unreachable!()
    }
    async fn get_profile(&self, id: &str) -> Result<UserProfileResponse, AppError> {
        Ok(UserProfileResponse {
            user_id: id.to_string(),
            username: "staff_user".to_string(),
            role_id: "role_staff".to_string(),
        })
    }
}

// Mock Role
struct MockRole;
#[async_trait]
impl RoleService for MockRole {
    async fn get_roles(&self) -> Result<Vec<RoleItemResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_role_detail(&self, id: &str) -> Result<RoleDetailResponse, AppError> {
        Ok(RoleDetailResponse {
            id: id.to_string(),
            name: "Mock Role".to_string(),
            is_active: true,
            created_at: chrono::Utc::now(),
            modify_at: chrono::Utc::now(),
            module_group: vec![],
        })
    }
    async fn create_role(&self, _req: CreateRoleRequest) -> Result<RoleItemResponse, AppError> {
        unreachable!()
    }
    async fn update_role(
        &self,
        _id: &str,
        _req: UpdateRoleRequest,
    ) -> Result<RoleItemResponse, AppError> {
        unreachable!()
    }
    async fn delete_role(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Role Permission
struct MockRolePerm;
#[async_trait]
impl RolePermissionService for MockRolePerm {
    async fn get_all_permissions(&self) -> Result<Vec<RolePermissionDto>, AppError> {
        Ok(vec![])
    }
    async fn get_permission_by_id(&self, _id: &str) -> Result<RolePermissionDto, AppError> {
        unreachable!()
    }
    async fn get_permissions_by_role(
        &self,
        _role_id: &str,
    ) -> Result<Vec<RolePermissionDto>, AppError> {
        Ok(vec![])
    }
    async fn create_permission(
        &self,
        _req: CreateRolePermissionRequest,
    ) -> Result<RolePermissionDto, AppError> {
        unreachable!()
    }
    async fn update_permission(
        &self,
        _id: &str,
        _req: UpdateRolePermissionRequest,
    ) -> Result<RolePermissionDto, AppError> {
        unreachable!()
    }
    async fn delete_permission(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock System Modules
struct MockSysModules;
#[async_trait]
impl SystemModulesService for MockSysModules {
    async fn get_module_groups(&self) -> Result<Vec<GroupResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_group_by_id(&self, _id: &str) -> Result<GroupResponse, AppError> {
        unreachable!()
    }
    async fn create_group(&self, _req: CreateGroupRequest) -> Result<GroupResponse, AppError> {
        unreachable!()
    }
    async fn update_group(
        &self,
        _id: &str,
        _req: UpdateGroupRequest,
    ) -> Result<GroupResponse, AppError> {
        unreachable!()
    }
    async fn delete_group(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
    async fn get_all_modules(&self) -> Result<Vec<ModuleTreeResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_module_by_id(&self, _id: &str) -> Result<ModuleTreeResponse, AppError> {
        unreachable!()
    }
    async fn create_module(
        &self,
        _req: CreateModuleRequest,
    ) -> Result<ModuleTreeResponse, AppError> {
        unreachable!()
    }
    async fn update_module(
        &self,
        _id: &str,
        _req: UpdateModuleRequest,
    ) -> Result<ModuleTreeResponse, AppError> {
        unreachable!()
    }
    async fn delete_module(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Plant
struct MockPlant;
#[async_trait]
impl PlantService for MockPlant {
    async fn list_plants(&self, _query: PlantListQuery) -> Result<Vec<PlantResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_plant(&self, _id: &str) -> Result<PlantResponse, AppError> {
        unreachable!()
    }
    async fn create_plant(
        &self,
        _req: CreatePlantRequest,
        _actor: &str,
    ) -> Result<PlantResponse, AppError> {
        unreachable!()
    }
    async fn update_plant(
        &self,
        _id: &str,
        _req: UpdatePlantRequest,
        _actor: &str,
    ) -> Result<PlantResponse, AppError> {
        unreachable!()
    }
    async fn delete_plant(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Location
struct MockLocation;
#[async_trait]
impl LocationService for MockLocation {
    async fn list_locations(
        &self,
        _query: LocationListQuery,
    ) -> Result<Vec<LocationResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_location(&self, _id: &str) -> Result<LocationResponse, AppError> {
        unreachable!()
    }
    async fn create_location(
        &self,
        _req: CreateLocationRequest,
        _actor: &str,
    ) -> Result<LocationResponse, AppError> {
        unreachable!()
    }
    async fn update_location(
        &self,
        _id: &str,
        _req: UpdateLocationRequest,
        _actor: &str,
    ) -> Result<LocationResponse, AppError> {
        unreachable!()
    }
    async fn delete_location(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Installation
struct MockInstallation;
#[async_trait]
impl InstallationService for MockInstallation {
    async fn list_installations(
        &self,
        _query: InstallationListQuery,
    ) -> Result<Vec<InstallationResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_installation(&self, _id: &str) -> Result<InstallationResponse, AppError> {
        unreachable!()
    }
    async fn create_installation(
        &self,
        _req: CreateInstallationRequest,
        _actor: &str,
    ) -> Result<InstallationResponse, AppError> {
        unreachable!()
    }
    async fn update_installation(
        &self,
        _id: &str,
        _req: UpdateInstallationRequest,
        _actor: &str,
    ) -> Result<InstallationResponse, AppError> {
        unreachable!()
    }
    async fn delete_installation(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock ItemCategory
struct MockItemCategory;
#[async_trait]
impl ItemCategoryService for MockItemCategory {
    async fn list_categories(
        &self,
        _query: ItemCategoryListQuery,
    ) -> Result<Vec<ItemCategoryResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_category(&self, _id: &str) -> Result<ItemCategoryResponse, AppError> {
        unreachable!()
    }
    async fn create_category(
        &self,
        _req: CreateItemCategoryRequest,
        _actor: &str,
    ) -> Result<ItemCategoryResponse, AppError> {
        unreachable!()
    }
    async fn update_category(
        &self,
        _id: &str,
        _req: UpdateItemCategoryRequest,
        _actor: &str,
    ) -> Result<ItemCategoryResponse, AppError> {
        unreachable!()
    }
    async fn delete_category(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Item
struct MockItem;
#[async_trait]
impl ItemService for MockItem {
    async fn list_items(&self, _query: ItemListQuery) -> Result<Vec<MasterItemResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_item(&self, _id: &str) -> Result<MasterItemResponse, AppError> {
        unreachable!()
    }
    async fn create_item(
        &self,
        _req: CreateMasterItemRequest,
        _actor: &str,
    ) -> Result<MasterItemResponse, AppError> {
        unreachable!()
    }
    async fn update_item(
        &self,
        _id: &str,
        _req: UpdateMasterItemRequest,
        _actor: &str,
    ) -> Result<MasterItemResponse, AppError> {
        unreachable!()
    }
    async fn delete_item(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Supplier
struct MockSupplier;
#[async_trait]
impl SupplierService for MockSupplier {
    async fn list_suppliers(
        &self,
        _query: SupplierListQuery,
    ) -> Result<Vec<SupplierResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_supplier(&self, _id: &str) -> Result<SupplierResponse, AppError> {
        unreachable!()
    }
    async fn create_supplier(
        &self,
        _req: CreateSupplierRequest,
        _actor: &str,
    ) -> Result<SupplierResponse, AppError> {
        unreachable!()
    }
    async fn update_supplier(
        &self,
        _id: &str,
        _req: UpdateSupplierRequest,
        _actor: &str,
    ) -> Result<SupplierResponse, AppError> {
        unreachable!()
    }
    async fn delete_supplier(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Customer
struct MockCustomer;
#[async_trait]
impl CustomerService for MockCustomer {
    async fn list_customers(
        &self,
        _query: CustomerListQuery,
    ) -> Result<Vec<CustomerResponse>, AppError> {
        Ok(vec![])
    }
    async fn get_customer(&self, _id: &str) -> Result<CustomerResponse, AppError> {
        unreachable!()
    }
    async fn create_customer(
        &self,
        _req: CreateCustomerRequest,
        _actor: &str,
    ) -> Result<CustomerResponse, AppError> {
        unreachable!()
    }
    async fn update_customer(
        &self,
        _id: &str,
        _req: UpdateCustomerRequest,
        _actor: &str,
    ) -> Result<CustomerResponse, AppError> {
        unreachable!()
    }
    async fn delete_customer(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Inventory
struct MockInventory;
#[async_trait]
impl InventoryService for MockInventory {
    async fn get_summary(
        &self,
        _query: InventorySummaryQuery,
    ) -> Result<InventorySummaryResponse, AppError> {
        Ok(InventorySummaryResponse {
            kpi: InventoryKpi {
                total_sku: 0,
                safe_count: 0,
                low_stock_count: 0,
                out_of_stock_count: 0,
                total_valuation_rp: rust_decimal::Decimal::ZERO,
            },
            items: vec![],
        })
    }
    async fn record_movement(
        &self,
        _req: CreateMovementRequest,
        _actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError> {
        unreachable!()
    }
    async fn record_stock_opname(
        &self,
        _req: CreateStockOpnameRequest,
        _actor: &str,
    ) -> Result<InventoryLedgerResponse, AppError> {
        unreachable!()
    }
    async fn get_stock_card(
        &self,
        _item_id: &str,
        _query: StockCardQuery,
    ) -> Result<StockCardResponse, AppError> {
        unreachable!()
    }
}

// Mock Session
struct MockSession;
#[async_trait]
impl SessionService for MockSession {
    async fn get_sessions(
        &self,
        _query: SessionListQuery,
    ) -> Result<SessionListResponse, AppError> {
        Ok(SessionListResponse {
            kpi: SessionKpi {
                active_sessions: 0,
                active_total_holes: 0,
                active_estimated_yield_kg: rust_decimal::Decimal::ZERO,
                completed_sessions: 0,
                unreported_today_count: 0,
            },
            items: vec![],
        })
    }
    async fn get_session_by_id(
        &self,
        _id: &str,
        _date: Option<chrono::NaiveDate>,
    ) -> Result<SessionDetailResponse, AppError> {
        unreachable!()
    }
    async fn create_session(
        &self,
        req: CreateSessionRequest,
        _user_id: &str,
    ) -> Result<SessionDetailResponse, AppError> {
        let now = chrono::Utc::now();
        Ok(SessionDetailResponse {
            item: agritech_api::modules::sessions::dto::SessionItemResponse {
                id: "ses_test_123".into(),
                session_name: req.session_name,
                plant_type_id: req.plant_type_id,
                plant_name: "Brokoli".into(),
                plant_variety: "Green Tower".into(),
                location_id: req.location_id,
                location_name: "Tonggoh Gudang".into(),
                installation_id: req.installation_id,
                installation_name: Some("Modul Aeroponik 1".into()),
                seeding_date: req.seeding_date,
                start_date: req.start_date,
                estimated_harvest_date: req.estimated_harvest_date.unwrap_or(req.start_date),
                harvest_date: None,
                total_holes: req.total_holes,
                status: agritech_api::modules::sessions::dto::SessionStatus::Active,
                is_locked: false,
                target_yield_kg: rust_decimal::Decimal::ZERO,
                actual_yield_kg: None,
                saleable_count: None,
                mortality_count: 0,
                total_daily_mortality: 0,
                yield_achievement_percent: None,
                survival_rate_percent: rust_decimal::Decimal::from(100),
                growth_info: agritech_api::modules::sessions::dto::GrowthInfo {
                    phase: agritech_api::modules::sessions::dto::GrowthPhase::Semai,
                    hst: 0,
                    seeding_days: 0,
                    active_week: 1,
                    week_index: 0,
                    progress_percent: rust_decimal::Decimal::ZERO,
                    is_overdue: false,
                },
                has_reported_today: false,
                latest_reading: None,
                notes: req.notes.unwrap_or_default(),
                created_at: now,
                created_by: "test-user-id".into(),
                modify_at: now,
                modify_by: "test-user-id".into(),
            },
            target_snapshot: agritech_api::modules::sessions::dto::SessionTargetSnapshot {
                target_growth_est_days: 35,
                target_veg_phase_days: 21,
                target_yield_per_hole: rust_decimal::Decimal::from(250),
                target_min_ph: rust_decimal::Decimal::from(5),
                target_ideal_min_ph: rust_decimal::Decimal::from(6),
                target_ideal_max_ph: rust_decimal::Decimal::from(7),
                target_max_ph: rust_decimal::Decimal::from(8),
                target_ppm_veg_weeks: vec![],
                target_ppm_gen_weeks: vec![],
                target_ideal_min_ppm_veg: 500,
                target_ideal_max_ppm_veg: 800,
                target_tol_min_ppm_veg: 400,
                target_tol_max_ppm_veg: 900,
                target_ideal_min_ppm_gen: 800,
                target_ideal_max_ppm_gen: 1200,
                target_tol_min_ppm_gen: 700,
                target_tol_max_ppm_gen: 1300,
                target_ideal_min_temp_env: rust_decimal::Decimal::from(20),
                target_ideal_max_temp_env: rust_decimal::Decimal::from(28),
                target_ideal_min_temp_water: rust_decimal::Decimal::from(18),
                target_ideal_max_temp_water: rust_decimal::Decimal::from(24),
                target_ideal_min_hum: rust_decimal::Decimal::from(60),
                target_ideal_max_hum: rust_decimal::Decimal::from(80),
            },
            active_targets: agritech_api::modules::daily_readings::dto::ActiveMetricTargets {
                ppm_ideal_min: rust_decimal::Decimal::from(500),
                ppm_ideal_max: rust_decimal::Decimal::from(800),
                ppm_tol_min: rust_decimal::Decimal::from(400),
                ppm_tol_max: rust_decimal::Decimal::from(900),
                ph_ideal_min: rust_decimal::Decimal::from(6),
                ph_ideal_max: rust_decimal::Decimal::from(7),
                ph_tol_min: rust_decimal::Decimal::from(5),
                ph_tol_max: rust_decimal::Decimal::from(8),
                temp_env_ideal_min: rust_decimal::Decimal::from(20),
                temp_env_ideal_max: rust_decimal::Decimal::from(28),
                temp_water_ideal_min: rust_decimal::Decimal::from(18),
                temp_water_ideal_max: rust_decimal::Decimal::from(24),
                hum_ideal_min: rust_decimal::Decimal::from(60),
                hum_ideal_max: rust_decimal::Decimal::from(80),
            },
            harvest_notes: "".into(),
        })
    }
    async fn update_session(
        &self,
        _id: &str,
        _req: UpdateSessionRequest,
        _user_id: &str,
    ) -> Result<SessionDetailResponse, AppError> {
        unreachable!()
    }
    async fn complete_harvest(
        &self,
        _id: &str,
        _req: CompleteHarvestRequest,
        _user_id: &str,
    ) -> Result<SessionDetailResponse, AppError> {
        unreachable!()
    }
    async fn delete_session(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Daily Reading
struct MockDailyReading;
#[async_trait]
impl DailyReadingService for MockDailyReading {
    async fn get_daily_readings_by_session(
        &self,
        _session_id: &str,
        _query: DailyReadingListQuery,
    ) -> Result<SessionDailyReadingsResponse, AppError> {
        unreachable!()
    }
    async fn create_daily_reading(
        &self,
        _session_id: &str,
        _req: CreateDailyReadingRequest,
        _user_id: &str,
    ) -> Result<DailyReadingItem, AppError> {
        unreachable!()
    }
    async fn delete_daily_reading(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
    async fn get_unreported_today(
        &self,
        _query: UnreportedTodayQuery,
    ) -> Result<UnreportedTodayResponse, AppError> {
        Ok(UnreportedTodayResponse {
            evaluation_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 22).unwrap(),
            total_active_sessions: 0,
            reported_count: 0,
            unreported_count: 0,
            unreported_sessions: vec![],
        })
    }
}

// Mock SOP
struct MockSop;
#[async_trait]
impl agritech_api::modules::plan::sop::service::SopService for MockSop {
    async fn get_sops(
        &self,
        _query: &agritech_api::modules::plan::sop::dto::SopQuery,
    ) -> Result<agritech_api::modules::plan::sop::dto::SopListResponse, AppError> {
        unreachable!()
    }
    async fn get_sop_by_id(
        &self,
        _id: &str,
    ) -> Result<agritech_api::modules::plan::sop::dto::SopResponse, AppError> {
        unreachable!()
    }
    async fn create_sop(
        &self,
        _req: agritech_api::modules::plan::sop::dto::CreateSopRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::plan::sop::dto::SopResponse, AppError> {
        unreachable!()
    }
    async fn update_sop(
        &self,
        _id: &str,
        _req: agritech_api::modules::plan::sop::dto::UpdateSopRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::plan::sop::dto::SopResponse, AppError> {
        unreachable!()
    }
    async fn delete_sop(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Task
struct MockTask;
#[async_trait]
impl agritech_api::modules::plan::tasks::service::TaskService for MockTask {
    async fn get_tasks(
        &self,
        _query: &agritech_api::modules::plan::tasks::dto::TaskQuery,
    ) -> Result<agritech_api::modules::plan::tasks::dto::TaskListResponse, AppError> {
        unreachable!()
    }
    async fn get_task_by_id(
        &self,
        _id: &str,
        _ref_date: Option<chrono::NaiveDate>,
    ) -> Result<agritech_api::modules::plan::tasks::dto::TaskResponse, AppError> {
        unreachable!()
    }
    async fn create_task(
        &self,
        _req: agritech_api::modules::plan::tasks::dto::CreateTaskRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::plan::tasks::dto::TaskResponse, AppError> {
        unreachable!()
    }
    async fn update_task(
        &self,
        _id: &str,
        _req: agritech_api::modules::plan::tasks::dto::UpdateTaskRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::plan::tasks::dto::TaskResponse, AppError> {
        unreachable!()
    }
    async fn update_task_status(
        &self,
        _id: &str,
        _req: agritech_api::modules::plan::tasks::dto::UpdateTaskStatusRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::plan::tasks::dto::TaskResponse, AppError> {
        unreachable!()
    }
    async fn delete_task(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Purchase
struct MockPurchase;
#[async_trait]
impl agritech_api::modules::finance::purchases::service::PurchaseService for MockPurchase {
    async fn get_purchases(
        &self,
        _query: &agritech_api::modules::finance::purchases::dto::PurchaseQuery,
    ) -> Result<agritech_api::modules::finance::purchases::dto::PurchaseListResponse, AppError>
    {
        Ok(
            agritech_api::modules::finance::purchases::dto::PurchaseListResponse {
                kpi: agritech_api::modules::finance::purchases::dto::PurchaseKpi {
                    total_purchases_count: 0,
                    total_spend_amount: rust_decimal::Decimal::ZERO,
                    direct_cost_total: rust_decimal::Decimal::ZERO,
                    shared_and_asset_cost_total: rust_decimal::Decimal::ZERO,
                },
                items: vec![],
            },
        )
    }
    async fn get_purchase_by_id(
        &self,
        _id: &str,
    ) -> Result<agritech_api::modules::finance::purchases::dto::PurchaseResponse, AppError> {
        unreachable!()
    }
    async fn create_purchase(
        &self,
        _req: agritech_api::modules::finance::purchases::dto::CreatePurchaseRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::finance::purchases::dto::PurchaseResponse, AppError> {
        unreachable!()
    }
    async fn delete_purchase(&self, _id: &str, _actor: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock Sale
struct MockSale;
#[async_trait]
impl agritech_api::modules::finance::sales::service::SaleService for MockSale {
    async fn get_sales(
        &self,
        _query: &agritech_api::modules::finance::sales::dto::SaleQuery,
    ) -> Result<agritech_api::modules::finance::sales::dto::SaleListResponse, AppError> {
        Ok(
            agritech_api::modules::finance::sales::dto::SaleListResponse {
                kpi: agritech_api::modules::finance::sales::dto::SaleKpi {
                    total_realized_revenue: rust_decimal::Decimal::ZERO,
                    total_receivable_amount: rust_decimal::Decimal::ZERO,
                    total_sold_kg: rust_decimal::Decimal::ZERO,
                    total_shipping_subsidy: rust_decimal::Decimal::ZERO,
                    order_pipeline_count: 0,
                },
                items: vec![],
            },
        )
    }
    async fn get_sale_by_id(
        &self,
        _id: &str,
    ) -> Result<agritech_api::modules::finance::sales::dto::SaleResponse, AppError> {
        unreachable!()
    }
    async fn create_sale(
        &self,
        _req: agritech_api::modules::finance::sales::dto::CreateSaleRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::finance::sales::dto::SaleResponse, AppError> {
        unreachable!()
    }
    async fn update_sale(
        &self,
        _id: &str,
        _req: agritech_api::modules::finance::sales::dto::UpdateSaleRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::finance::sales::dto::SaleResponse, AppError> {
        unreachable!()
    }
    async fn delete_sale(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

// Mock HPP
struct MockHpp;
#[async_trait]
impl agritech_api::modules::finance::hpp::service::HppService for MockHpp {
    async fn get_hpp_list(
        &self,
        _query: &agritech_api::modules::finance::hpp::dto::HppListQuery,
    ) -> Result<agritech_api::modules::finance::hpp::dto::HppListResponse, AppError> {
        Ok(agritech_api::modules::finance::hpp::dto::HppListResponse {
            kpi: agritech_api::modules::finance::hpp::dto::HppSummaryKpi {
                total_cumulative_hpp: rust_decimal::Decimal::ZERO,
                total_cumulative_revenue: rust_decimal::Decimal::ZERO,
                total_cumulative_shipping_subsidy: rust_decimal::Decimal::ZERO,
                total_cumulative_net_profit: rust_decimal::Decimal::ZERO,
                average_profit_margin_percent: rust_decimal::Decimal::ZERO,
                profitable_sessions_count: 0,
                loss_sessions_count: 0,
            },
            items: vec![],
        })
    }
    async fn get_session_rhpp(
        &self,
        _session_id: &str,
        _eval_date: Option<chrono::NaiveDate>,
    ) -> Result<agritech_api::modules::finance::hpp::dto::RHPPDetailResponse, AppError> {
        unreachable!()
    }
    async fn lock_session_snapshot(
        &self,
        _session_id: &str,
        _actor: &str,
    ) -> Result<agritech_api::modules::finance::hpp::dto::RHPPDetailResponse, AppError> {
        unreachable!()
    }
}

// Mock Tax
struct MockTax;
#[async_trait]
impl agritech_api::modules::finance::tax::service::TaxService for MockTax {
    async fn get_dashboard(
        &self,
        _query: &agritech_api::modules::finance::tax::dto::TaxDashboardQuery,
    ) -> Result<agritech_api::modules::finance::tax::dto::TaxDashboardData, AppError> {
        Ok(agritech_api::modules::finance::tax::dto::TaxDashboardData {
            settings: agritech_api::modules::finance::tax::dto::TaxSettingsDto {
                id: "tax_cfg_default".into(),
                scheme_name: "PPh Final UMKM".into(),
                rate: rust_decimal::Decimal::from_str("0.0050").unwrap(),
                rate_percent: rust_decimal::Decimal::from_str("0.50").unwrap(),
                description: "Test".into(),
                modify_at: chrono::Utc::now(),
                modify_by: "system".into(),
            },
            current_period: agritech_api::modules::finance::tax::dto::TaxCurrentPeriodDto {
                report_id: None,
                period_month: 11,
                period_year: 2026,
                total_revenue: rust_decimal::Decimal::ZERO,
                realized_sales_count: 0,
                tax_rate: rust_decimal::Decimal::from_str("0.0050").unwrap(),
                tax_rate_percent: rust_decimal::Decimal::from_str("0.50").unwrap(),
                estimated_tax_amount: rust_decimal::Decimal::ZERO,
                status: agritech_api::modules::finance::tax::dto::TaxStatus::Unpaid,
                deadline_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 15).unwrap(),
                days_remaining: 19,
                alert_status: agritech_api::modules::finance::tax::dto::TaxAlertStatus::Normal,
                payment_date: None,
                billing_code: "".into(),
                notes: "".into(),
            },
            tax_history: vec![],
        })
    }
    async fn pay_tax(
        &self,
        _req: agritech_api::modules::finance::tax::dto::PayTaxRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::finance::tax::dto::TaxReportItem, AppError> {
        unreachable!()
    }
    async fn update_settings(
        &self,
        _req: agritech_api::modules::finance::tax::dto::UpdateTaxSettingsRequest,
        _actor: &str,
    ) -> Result<agritech_api::modules::finance::tax::dto::TaxSettingsDto, AppError> {
        unreachable!()
    }
    async fn delete_report(&self, _id: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

struct MockSummary;
#[async_trait]
impl agritech_api::modules::reports::summary::service::ReportSummaryService for MockSummary {
    async fn get_cashflow_summary(
        &self,
        _query: &agritech_api::modules::reports::summary::dto::CashflowSummaryQuery,
    ) -> Result<agritech_api::modules::reports::summary::dto::CashflowSummaryResponse, AppError>
    {
        unreachable!()
    }
}

struct MockAudit;
#[async_trait]
impl agritech_api::modules::reports::audit::service::AuditService for MockAudit {
    async fn get_logs(
        &self,
        _query: &agritech_api::modules::reports::audit::dto::AuditLogQuery,
    ) -> Result<agritech_api::modules::reports::audit::dto::AuditLogListResponse, AppError> {
        unreachable!()
    }
    async fn get_modules(&self) -> Result<Vec<String>, AppError> {
        unreachable!()
    }
    async fn record_event(
        &self,
        _action: agritech_api::modules::reports::audit::dto::AuditAction,
        _module: &str,
        _user_id: Option<String>,
        _username: &str,
        _target_id: &str,
        _description: &str,
        _old_data: Option<serde_json::Value>,
        _new_data: Option<serde_json::Value>,
        _ip_address: Option<String>,
    ) -> Result<(), AppError> {
        Ok(())
    }
}

struct MockAiAudit;
#[async_trait]
impl agritech_api::modules::reports::ai_audit::service::AiAuditService for MockAiAudit {
    async fn scan(
        &self,
        _query: &agritech_api::modules::reports::ai_audit::dto::AiAuditScanQuery,
    ) -> Result<agritech_api::modules::reports::ai_audit::dto::AiAuditScanResponse, AppError> {
        unreachable!()
    }
}

fn build_test_state() -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:5432/agritech_test")
        .expect("Lazy pool creation should not fail");

    let config = AppConfig {
        port: 8080,
        host: "127.0.0.1".into(),
        db_port: 5432,
        db_user: "postgres".into(),
        db_pass: "password".into(),
        db_name: "agritech_test".into(),
        database_url: "postgres://postgres:password@127.0.0.1:5432/agritech_test".into(),
        jwt_secret: "test_secret_minimum_32_characters_for_testing".into(),
        jwt_expiration_hours: 24,
        env: "test".into(),
        enable_swagger: true,
    };

    AppState {
        db: pool,
        config: Arc::new(config),
        health_service: Arc::new(MockHealth),
        auth_service: Arc::new(MockAuth),
        user_service: Arc::new(MockUser),
        role_service: Arc::new(MockRole),
        role_permission_service: Arc::new(MockRolePerm),
        system_modules_service: Arc::new(MockSysModules),
        plant_service: Arc::new(MockPlant),
        location_service: Arc::new(MockLocation),
        installation_service: Arc::new(MockInstallation),
        item_category_service: Arc::new(MockItemCategory),
        item_service: Arc::new(MockItem),
        supplier_service: Arc::new(MockSupplier),
        customer_service: Arc::new(MockCustomer),
        inventory_service: Arc::new(MockInventory),
        session_service: Arc::new(MockSession),
        daily_reading_service: Arc::new(MockDailyReading),
        sop_service: Arc::new(MockSop),
        task_service: Arc::new(MockTask),
        purchase_service: Arc::new(MockPurchase),
        sale_service: Arc::new(MockSale),
        hpp_service: Arc::new(MockHpp),
        tax_service: Arc::new(MockTax),
        report_summary_service: Arc::new(MockSummary),
        audit_service: Arc::new(MockAudit),
        ai_audit_service: Arc::new(MockAiAudit),
    }
}

#[tokio::test]
async fn test_integration_health_check_envelope() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 200);
    assert_eq!(json["message"], "Service is healthy");
    assert_eq!(json["data"]["app"], "OK");
    assert_eq!(json["data"]["database"], "OK");
}

#[tokio::test]
async fn test_integration_unauthenticated_protected_route() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/api/v1/roles")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 401);
    assert_eq!(json["message"], "Unauthorized");
}

#[tokio::test]
async fn test_integration_unauthenticated_users_route() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/api/v1/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 401);
    assert_eq!(json["message"], "Unauthorized");
}

#[tokio::test]
async fn test_integration_authenticated_via_cookie_token() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req = Request::builder()
        .uri("/api/v1/roles")
        .header(
            axum::http::header::COOKIE,
            format!("other_cookie=xyz; token={token}; theme=emerald"),
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 200);
}

#[tokio::test]
async fn test_integration_authenticated_via_bearer_header() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req = Request::builder()
        .uri("/api/v1/users")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 200);
}

#[tokio::test]
async fn test_integration_swagger_ui_with_emerald_theme() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/swagger-ui/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Verify emerald theme is embedded directly in Swagger HTML
    assert!(html.contains("Dark Emerald"));
    assert!(html.contains("#0B1121"));
}

#[tokio::test]
async fn test_integration_swagger_custom_css_endpoint() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/swagger-ui/custom.css")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let css = String::from_utf8(body.to_vec()).unwrap();

    assert!(css.contains(".swagger-ui"));
    assert!(css.contains("#10B981"));
}

#[tokio::test]
async fn test_integration_openapi_spec_json() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/api-docs/openapi.json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        !json_str.contains("/components/schemas/T\""),
        "OpenAPI spec must not contain unresolvable pointer /components/schemas/T"
    );
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["info"]["title"], "AgriTech API");
    assert_eq!(json["info"]["version"], "1.0.0");
}

#[tokio::test]
async fn test_integration_fallback_404_envelope() {
    let app = create_app(build_test_state());

    let req = Request::builder()
        .uri("/api/v1/invalid-route-xyz")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 404);
    assert_eq!(json["message"], "Not Found");
}

#[tokio::test]
async fn test_integration_masters_plants_protected_and_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    // 1. Unauthenticated -> 401
    let req_unauth = Request::builder()
        .uri("/api/v1/masters/plants")
        .body(Body::empty())
        .unwrap();
    let res_unauth = app.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Authenticated -> 200
    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req_auth = Request::builder()
        .uri("/api/v1/masters/plants")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth = app.oneshot(req_auth).await.unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);

    let body = res_auth.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 200);
    assert_eq!(json["message"], "Successfully retrieved plants");
}

#[tokio::test]
async fn test_integration_masters_locations_and_installations_protected() {
    let app = create_app(build_test_state());

    let req_loc = Request::builder()
        .uri("/api/v1/masters/locations")
        .body(Body::empty())
        .unwrap();
    let res_loc = app.clone().oneshot(req_loc).await.unwrap();
    assert_eq!(res_loc.status(), StatusCode::UNAUTHORIZED);

    let req_ins = Request::builder()
        .uri("/api/v1/masters/installations")
        .body(Body::empty())
        .unwrap();
    let res_ins = app.oneshot(req_ins).await.unwrap();
    assert_eq!(res_ins.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_integration_masters_item_categories_and_items_protected_and_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    // Unauthenticated checks
    let req_unauth_cat = Request::builder()
        .uri("/api/v1/masters/item-categories")
        .body(Body::empty())
        .unwrap();
    let res_unauth_cat = app.clone().oneshot(req_unauth_cat).await.unwrap();
    assert_eq!(res_unauth_cat.status(), StatusCode::UNAUTHORIZED);

    let req_unauth_itm = Request::builder()
        .uri("/api/v1/masters/items")
        .body(Body::empty())
        .unwrap();
    let res_unauth_itm = app.clone().oneshot(req_unauth_itm).await.unwrap();
    assert_eq!(res_unauth_itm.status(), StatusCode::UNAUTHORIZED);

    // Authenticated checks
    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req_auth_cat = Request::builder()
        .uri("/api/v1/masters/item-categories")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth_cat = app.clone().oneshot(req_auth_cat).await.unwrap();
    assert_eq!(res_auth_cat.status(), StatusCode::OK);

    let body_cat = res_auth_cat.into_body().collect().await.unwrap().to_bytes();
    let json_cat: serde_json::Value = serde_json::from_slice(&body_cat).unwrap();
    assert_eq!(json_cat["status"], 200);
    assert_eq!(
        json_cat["message"],
        "Successfully retrieved item categories"
    );

    let req_auth_itm = Request::builder()
        .uri("/api/v1/masters/items")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth_itm = app.oneshot(req_auth_itm).await.unwrap();
    assert_eq!(res_auth_itm.status(), StatusCode::OK);

    let body_itm = res_auth_itm.into_body().collect().await.unwrap().to_bytes();
    let json_itm: serde_json::Value = serde_json::from_slice(&body_itm).unwrap();
    assert_eq!(json_itm["status"], 200);
    assert_eq!(json_itm["message"], "Successfully retrieved items");
}

#[tokio::test]
async fn test_integration_masters_suppliers_and_customers_protected_and_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    // Unauthenticated checks
    let req_unauth_sup = Request::builder()
        .uri("/api/v1/masters/suppliers")
        .body(Body::empty())
        .unwrap();
    let res_unauth_sup = app.clone().oneshot(req_unauth_sup).await.unwrap();
    assert_eq!(res_unauth_sup.status(), StatusCode::UNAUTHORIZED);

    let req_unauth_cus = Request::builder()
        .uri("/api/v1/masters/customers")
        .body(Body::empty())
        .unwrap();
    let res_unauth_cus = app.clone().oneshot(req_unauth_cus).await.unwrap();
    assert_eq!(res_unauth_cus.status(), StatusCode::UNAUTHORIZED);

    // Authenticated checks
    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req_auth_sup = Request::builder()
        .uri("/api/v1/masters/suppliers")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth_sup = app.clone().oneshot(req_auth_sup).await.unwrap();
    assert_eq!(res_auth_sup.status(), StatusCode::OK);

    let body_sup = res_auth_sup.into_body().collect().await.unwrap().to_bytes();
    let json_sup: serde_json::Value = serde_json::from_slice(&body_sup).unwrap();
    assert_eq!(json_sup["status"], 200);
    assert_eq!(json_sup["message"], "Successfully retrieved suppliers");

    let req_auth_cus = Request::builder()
        .uri("/api/v1/masters/customers")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth_cus = app.oneshot(req_auth_cus).await.unwrap();
    assert_eq!(res_auth_cus.status(), StatusCode::OK);

    let body_cus = res_auth_cus.into_body().collect().await.unwrap().to_bytes();
    let json_cus: serde_json::Value = serde_json::from_slice(&body_cus).unwrap();
    assert_eq!(json_cus["status"], 200);
    assert_eq!(json_cus["message"], "Successfully retrieved customers");
}

#[tokio::test]
async fn test_integration_inventory_summary_protected_and_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    // 1. Unauthenticated -> 401
    let req_unauth = Request::builder()
        .uri("/api/v1/inventory/summary")
        .body(Body::empty())
        .unwrap();
    let res_unauth = app.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Authenticated -> 200
    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req_auth = Request::builder()
        .uri("/api/v1/inventory/summary")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth = app.oneshot(req_auth).await.unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);

    let body = res_auth.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 200);
    assert_eq!(json["message"], "Successfully retrieved inventory summary");
}

#[tokio::test]
async fn test_integration_sessions_protected_and_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    // 1. Unauthenticated -> 401
    let req_unauth = Request::builder()
        .uri("/api/v1/sessions")
        .body(Body::empty())
        .unwrap();
    let res_unauth = app.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Authenticated -> 200
    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req_auth = Request::builder()
        .uri("/api/v1/sessions")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth = app.oneshot(req_auth).await.unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);

    let body = res_auth.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 200);
    assert_eq!(json["message"], "Successfully retrieved planting sessions");
}

#[tokio::test]
async fn test_integration_daily_readings_unreported_today_protected_and_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    // 1. Unauthenticated -> 401
    let req_unauth = Request::builder()
        .uri("/api/v1/daily-readings/unreported-today")
        .body(Body::empty())
        .unwrap();
    let res_unauth = app.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Authenticated -> 200
    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req_auth = Request::builder()
        .uri("/api/v1/daily-readings/unreported-today")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth = app.oneshot(req_auth).await.unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);

    let body = res_auth.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 200);
    assert_eq!(
        json["message"],
        "Successfully retrieved unreported active sessions"
    );
}

#[tokio::test]
async fn test_integration_create_session_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "testuser".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    // Sesuai payload dari CreateSessionModal (screenshot user)
    let payload = serde_json::json!({
        "session_name": "testing",
        "plant_type_id": "plt_test_1",
        "location_id": "loc_test_1",
        "installation_id": "ins_test_1",
        "seeding_date": "2026-11-16",
        "start_date": "2026-11-22",
        "estimated_harvest_date": "2026-12-27",
        "total_holes": 500,
        "notes": ""
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/sessions")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], 201);
    assert_eq!(json["message"], "Successfully created planting session");
    assert_eq!(json["data"]["session_name"], "testing");
    assert_eq!(json["data"]["total_holes"], 500);
}

#[tokio::test]
async fn test_integration_register_unauthenticated_returns_401() {
    let state = build_test_state();
    let app = create_app(state);

    let payload = serde_json::json!({
        "name": "QA Attacker",
        "email": "attacker@example.com",
        "username": "attacker",
        "password": "Password123!",
        "role_id": "role_superadmin"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_integration_register_unauthorized_user_returns_403() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "test-user-id".into(),
        username: "unauthorized_user".into(),
        role_id: "forbidden_role".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let payload = serde_json::json!({
        "name": "QA Attacker",
        "email": "attacker@example.com",
        "username": "attacker",
        "password": "Password123!",
        "role_id": "role_superadmin"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_integration_inactive_user_rejected_with_401() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "inactive-user-id".into(),
        username: "inactive_user".into(),
        role_id: "some_role".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/users/profile")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_integration_rbac_forbidden_user_returns_403() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "user-id".into(),
        username: "unauthorized_user".into(),
        role_id: "forbidden_role".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/sessions")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_integration_profile_exception_allows_active_non_admin() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();
    let app = create_app(state);

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "staff-user-id".into(),
        username: "staff_user".into(),
        role_id: "role_staff".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/users/profile")
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_integration_rbac_nested_routes_forbidden_for_unauthorized_role() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "forbidden-user-id".into(),
        username: "unauthorized_user".into(),
        role_id: "forbidden_role".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let endpoints = vec![
        ("GET", "/api/v1/users"),
        ("GET", "/api/v1/masters/users"),
        ("POST", "/api/v1/roles"),
        ("POST", "/api/v1/masters/roles"),
        ("GET", "/api/v1/masters/plants"),
        ("GET", "/api/v1/inventory/summary"),
        ("GET", "/api/v1/production/inventory/summary"),
        ("GET", "/api/v1/sessions"),
        ("GET", "/api/v1/production/sessions"),
        ("GET", "/api/v1/sessions/ses_mock/daily-readings"),
        ("GET", "/api/v1/production/sessions/ses_mock/daily-readings"),
        ("GET", "/api/v1/daily-readings/unreported-today"),
        ("GET", "/api/v1/production/daily-readings/unreported-today"),
        ("GET", "/api/v1/system-modules/groups"),
        ("GET", "/api/v1/masters/system-modules/groups"),
        ("GET", "/api/v1/roles/other_role_id"),
        ("GET", "/api/v1/masters/roles/other_role_id"),
    ];

    for (method, uri) in endpoints {
        let app = create_app(state.clone());
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "Expected 403 Forbidden for nested route {method} {uri}"
        );
    }
}

#[tokio::test]
async fn test_integration_rbac_exempt_routes_allowed_for_unauthorized_role() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "forbidden-user-id".into(),
        username: "unauthorized_user".into(),
        role_id: "forbidden_role".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let exempt_endpoints = vec![
        "/api/v1/users/profile",
        "/api/v1/masters/users/profile",
        "/api/v1/roles/forbidden_role",
        "/api/v1/masters/roles/forbidden_role",
        "/api/v1/roles/rolePermissions/role/forbidden_role",
        "/api/v1/masters/roles/rolePermissions/role/forbidden_role",
    ];

    for uri in exempt_endpoints {
        let app = create_app(state.clone());
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Expected 200 OK for exempt route GET {uri}"
        );
    }
}

#[tokio::test]
async fn test_integration_production_canonical_routes_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "user-1".into(),
        username: "operator".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let canonical_endpoints = vec![
        "/api/v1/production/sessions",
        "/api/v1/production/inventory/summary",
        "/api/v1/production/daily-readings/unreported-today",
    ];

    for uri in canonical_endpoints {
        let app = create_app(state.clone());
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Expected 200 OK for canonical production route GET {uri}"
        );
    }
}

#[tokio::test]
async fn test_integration_finance_canonical_routes_authenticated() {
    let state = build_test_state();
    let jwt_secret = state.config.jwt_secret.clone();

    let now = chrono::Utc::now().timestamp();
    let claims = agritech_api::middleware::auth::Claims {
        user_id: "user-1".into(),
        username: "admin".into(),
        role_id: "role-1".into(),
        exp: now + 3600,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap();

    let endpoints = vec![
        "/api/v1/finance/purchases",
        "/api/v1/purchases",
        "/api/v1/finance/sales",
        "/api/v1/sales",
        "/api/v1/finance/hpp",
        "/api/v1/finance/tax/dashboard",
    ];

    for uri in endpoints {
        let app = create_app(state.clone());
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Expected 200 OK for finance route GET {uri}"
        );
    }
}
