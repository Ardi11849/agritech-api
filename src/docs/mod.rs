pub mod theme;

use std::sync::Arc;

use axum::http::{header, StatusCode};
use axum::Router;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::Config;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "Enter JWT bearer token in format: Bearer <your-jwt-token>",
                        ))
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::modules::health::handler::check_health,
        crate::modules::auth::handler::register,
        crate::modules::auth::handler::login,
        crate::modules::user::handler::get_profile,
        crate::modules::user::handler::get_users,
        crate::modules::user::handler::get_user,
        crate::modules::user::handler::create_user,
        crate::modules::user::handler::update_user,
        crate::modules::user::handler::delete_user,
        crate::modules::role::handler::get_roles,
        crate::modules::role::handler::get_role,
        crate::modules::role::handler::create_role,
        crate::modules::role::handler::update_role,
        crate::modules::role::handler::delete_role,
        crate::modules::role::handler::get_all_permissions,
        crate::modules::role::handler::get_permission_by_id,
        crate::modules::role::handler::get_permissions_by_role,
        crate::modules::role::handler::create_permission,
        crate::modules::role::handler::update_permission,
        crate::modules::role::handler::delete_permission,
        crate::modules::system_modules::handler::get_groups,
        crate::modules::system_modules::handler::get_group_by_id,
        crate::modules::system_modules::handler::create_group,
        crate::modules::system_modules::handler::update_group,
        crate::modules::system_modules::handler::delete_group,
        crate::modules::system_modules::handler::get_modules,
        crate::modules::system_modules::handler::get_module_by_id,
        crate::modules::system_modules::handler::create_module,
        crate::modules::system_modules::handler::update_module,
        crate::modules::system_modules::handler::delete_module,
        crate::modules::masters::plants::handler::list_plants,
        crate::modules::masters::plants::handler::get_plant,
        crate::modules::masters::plants::handler::create_plant,
        crate::modules::masters::plants::handler::update_plant,
        crate::modules::masters::plants::handler::delete_plant,
        crate::modules::masters::locations::handler::list_locations,
        crate::modules::masters::locations::handler::get_location,
        crate::modules::masters::locations::handler::create_location,
        crate::modules::masters::locations::handler::update_location,
        crate::modules::masters::locations::handler::delete_location,
        crate::modules::masters::installations::handler::list_installations,
        crate::modules::masters::installations::handler::get_installation,
        crate::modules::masters::installations::handler::create_installation,
        crate::modules::masters::installations::handler::update_installation,
        crate::modules::masters::installations::handler::delete_installation,
        crate::modules::masters::item_categories::handler::list_categories,
        crate::modules::masters::item_categories::handler::get_category,
        crate::modules::masters::item_categories::handler::create_category,
        crate::modules::masters::item_categories::handler::update_category,
        crate::modules::masters::item_categories::handler::delete_category,
        crate::modules::masters::items::handler::list_items,
        crate::modules::masters::items::handler::get_item,
        crate::modules::masters::items::handler::create_item,
        crate::modules::masters::items::handler::update_item,
        crate::modules::masters::items::handler::delete_item,
        crate::modules::masters::suppliers::handler::list_suppliers,
        crate::modules::masters::suppliers::handler::get_supplier,
        crate::modules::masters::suppliers::handler::create_supplier,
        crate::modules::masters::suppliers::handler::update_supplier,
        crate::modules::masters::suppliers::handler::delete_supplier,
        crate::modules::masters::customers::handler::list_customers,
        crate::modules::masters::customers::handler::get_customer,
        crate::modules::masters::customers::handler::create_customer,
        crate::modules::masters::customers::handler::update_customer,
        crate::modules::masters::customers::handler::delete_customer,
        crate::modules::inventory::handler::get_summary,
        crate::modules::inventory::handler::record_movement,
        crate::modules::inventory::handler::record_stock_opname,
        crate::modules::inventory::handler::get_stock_card,
        crate::modules::sessions::handler::list_sessions,
        crate::modules::sessions::handler::get_session,
        crate::modules::sessions::handler::create_session,
        crate::modules::sessions::handler::update_session,
        crate::modules::sessions::handler::complete_harvest,
        crate::modules::sessions::handler::delete_session,
        crate::modules::daily_readings::handler::list_session_daily_readings,
        crate::modules::daily_readings::handler::create_session_daily_reading,
        crate::modules::daily_readings::handler::delete_daily_reading,
        crate::modules::daily_readings::handler::get_unreported_today,
        crate::modules::plan::sop::handler::list_sops,
        crate::modules::plan::sop::handler::get_sop,
        crate::modules::plan::sop::handler::create_sop,
        crate::modules::plan::sop::handler::update_sop,
        crate::modules::plan::sop::handler::delete_sop,
        crate::modules::plan::tasks::handler::list_tasks,
        crate::modules::plan::tasks::handler::get_task,
        crate::modules::plan::tasks::handler::create_task,
        crate::modules::plan::tasks::handler::update_task,
        crate::modules::plan::tasks::handler::update_task_status,
        crate::modules::plan::tasks::handler::delete_task,
        crate::modules::finance::purchases::handler::get_purchases,
        crate::modules::finance::purchases::handler::get_purchase,
        crate::modules::finance::purchases::handler::create_purchase,
        crate::modules::finance::purchases::handler::delete_purchase,
        crate::modules::finance::sales::handler::get_sales,
        crate::modules::finance::sales::handler::get_sale,
        crate::modules::finance::sales::handler::create_sale,
        crate::modules::finance::sales::handler::update_sale,
        crate::modules::finance::sales::handler::delete_sale,
        crate::modules::finance::hpp::handler::get_hpp_list,
        crate::modules::finance::hpp::handler::get_session_rhpp,
        crate::modules::finance::hpp::handler::lock_snapshot,
        crate::modules::finance::tax::handler::get_tax_dashboard,
        crate::modules::finance::tax::handler::pay_tax,
        crate::modules::finance::tax::handler::update_tax_settings,
        crate::modules::finance::tax::handler::delete_tax_report,
    ),
    components(
        schemas(
            crate::common::response::ApiResponse<serde_json::Value>,
            crate::modules::health::dto::HealthStatus,
            crate::modules::health::dto::ComponentStatus,
            crate::modules::auth::dto::RegisterRequest,
            crate::modules::auth::dto::RegisterResponse,
            crate::modules::auth::dto::LoginRequest,
            crate::modules::auth::dto::LoginResponse,
            crate::modules::auth::dto::LoginUserResponse,
            crate::modules::user::dto::UserProfileResponse,
            crate::modules::user::dto::UserItemResponse,
            crate::modules::user::dto::CreateUserRequest,
            crate::modules::user::dto::UpdateUserRequest,
            crate::modules::role::dto::CreateRoleRequest,
            crate::modules::role::dto::UpdateRoleRequest,
            crate::modules::role::dto::RoleItemResponse,
            crate::modules::role::dto::RoleDetailResponse,
            crate::modules::role::dto::ModuleGroupDetailDto,
            crate::modules::role::dto::ModuleDetailDto,
            crate::modules::role::dto::RolePermissionDto,
            crate::modules::role::dto::CreateRolePermissionRequest,
            crate::modules::role::dto::UpdateRolePermissionRequest,
            crate::modules::system_modules::dto::CreateGroupRequest,
            crate::modules::system_modules::dto::UpdateGroupRequest,
            crate::modules::system_modules::dto::GroupResponse,
            crate::modules::system_modules::dto::CreateModuleRequest,
            crate::modules::system_modules::dto::UpdateModuleRequest,
            crate::modules::system_modules::dto::ModuleTreeResponse,
            crate::modules::masters::plants::dto::PpmWeekRange,
            crate::modules::masters::plants::dto::CreatePlantRequest,
            crate::modules::masters::plants::dto::PlantResponse,
            crate::modules::masters::locations::dto::CreateLocationRequest,
            crate::modules::masters::locations::dto::LocationResponse,
            crate::modules::masters::installations::dto::InstallationStatus,
            crate::modules::masters::installations::dto::CreateInstallationRequest,
            crate::modules::masters::installations::dto::InstallationResponse,
            crate::modules::masters::item_categories::dto::CreateItemCategoryRequest,
            crate::modules::masters::item_categories::dto::UpdateItemCategoryRequest,
            crate::modules::masters::item_categories::dto::ItemCategoryResponse,
            crate::modules::masters::items::dto::StockHealthStatus,
            crate::modules::masters::items::dto::CreateMasterItemRequest,
            crate::modules::masters::items::dto::UpdateMasterItemRequest,
            crate::modules::masters::items::dto::MasterItemResponse,
            crate::modules::masters::suppliers::dto::CreateSupplierRequest,
            crate::modules::masters::suppliers::dto::UpdateSupplierRequest,
            crate::modules::masters::suppliers::dto::SupplierResponse,
            crate::modules::masters::customers::dto::CreateCustomerRequest,
            crate::modules::masters::customers::dto::UpdateCustomerRequest,
            crate::modules::masters::customers::dto::CustomerResponse,
            crate::modules::inventory::dto::MovementType,
            crate::modules::inventory::dto::StockMovementInputType,
            crate::modules::inventory::dto::InventoryKpi,
            crate::modules::inventory::dto::InventoryItemSummary,
            crate::modules::inventory::dto::InventorySummaryResponse,
            crate::modules::inventory::dto::CreateMovementRequest,
            crate::modules::inventory::dto::CreateStockOpnameRequest,
            crate::modules::inventory::dto::InventoryLedgerResponse,
            crate::modules::inventory::dto::StockCardResponse,
            crate::modules::sessions::dto::SessionStatus,
            crate::modules::sessions::dto::GrowthPhase,
            crate::modules::sessions::dto::GrowthInfo,
            crate::modules::sessions::dto::LatestReadingSummary,
            crate::modules::sessions::dto::SessionKpi,
            crate::modules::sessions::dto::SessionItemResponse,
            crate::modules::sessions::dto::SessionTargetSnapshot,
            crate::modules::sessions::dto::SessionDetailResponse,
            crate::modules::sessions::dto::SessionListResponse,
            crate::modules::sessions::dto::CreateSessionRequest,
            crate::modules::sessions::dto::UpdateSessionRequest,
            crate::modules::sessions::dto::CompleteHarvestRequest,
            crate::modules::daily_readings::dto::MetricStatus,
            crate::modules::daily_readings::dto::ActiveMetricTargets,
            crate::modules::daily_readings::dto::DailyReadingDiagnostics,
            crate::modules::daily_readings::dto::DailyReadingItem,
            crate::modules::daily_readings::dto::CreateDailyReadingRequest,
            crate::modules::daily_readings::dto::SessionDailyReadingsHeader,
            crate::modules::daily_readings::dto::SessionDailyReadingsResponse,
            crate::modules::daily_readings::dto::UnreportedTodayResponse,
            crate::modules::plan::sop::dto::SopCategory,
            crate::modules::plan::sop::dto::SopStatus,
            crate::modules::plan::sop::dto::SopStepInput,
            crate::modules::plan::sop::dto::SopStepResponse,
            crate::modules::plan::sop::dto::SopResponse,
            crate::modules::plan::sop::dto::SopKpi,
            crate::modules::plan::sop::dto::SopListResponse,
            crate::modules::plan::sop::dto::CreateSopRequest,
            crate::modules::plan::sop::dto::UpdateSopRequest,
            crate::modules::plan::tasks::dto::TaskStatus,
            crate::modules::plan::tasks::dto::TaskPriority,
            crate::modules::plan::tasks::dto::TaskResponse,
            crate::modules::plan::tasks::dto::TaskKpi,
            crate::modules::plan::tasks::dto::TaskListResponse,
            crate::modules::plan::tasks::dto::CreateTaskRequest,
            crate::modules::plan::tasks::dto::UpdateTaskRequest,
            crate::modules::plan::tasks::dto::UpdateTaskStatusRequest,
            crate::modules::finance::purchases::dto::AllocationType,
            crate::modules::finance::purchases::dto::CreatePurchaseItemRequest,
            crate::modules::finance::purchases::dto::CreatePurchaseRequest,
            crate::modules::finance::purchases::dto::PurchaseItemResponse,
            crate::modules::finance::purchases::dto::PurchaseResponse,
            crate::modules::finance::purchases::dto::PurchaseKpi,
            crate::modules::finance::purchases::dto::PurchaseListResponse,
            crate::modules::finance::sales::dto::SaleStatus,
            crate::modules::finance::sales::dto::PaymentStatus,
            crate::modules::finance::sales::dto::ShippingBorneBy,
            crate::modules::finance::sales::dto::CreateSaleRequest,
            crate::modules::finance::sales::dto::SaleResponse,
            crate::modules::finance::sales::dto::SaleKpi,
            crate::modules::finance::sales::dto::SaleListResponse,
            crate::modules::finance::hpp::dto::HppSummaryKpi,
            crate::modules::finance::hpp::dto::HppSessionItem,
            crate::modules::finance::hpp::dto::HppListResponse,
            crate::modules::finance::hpp::dto::HppSnapshotInfo,
            crate::modules::finance::hpp::dto::RHPPDirectPurchaseItem,
            crate::modules::finance::hpp::dto::RHPPAllocatedPurchaseItem,
            crate::modules::finance::hpp::dto::RHPPRealizedSaleItem,
            crate::modules::finance::hpp::dto::RHPPDetailResponse,
            crate::modules::finance::tax::dto::TaxStatus,
            crate::modules::finance::tax::dto::TaxAlertStatus,
            crate::modules::finance::tax::dto::TaxSettingsDto,
            crate::modules::finance::tax::dto::TaxCurrentPeriodDto,
            crate::modules::finance::tax::dto::TaxReportItem,
            crate::modules::finance::tax::dto::TaxDashboardData,
            crate::modules::finance::tax::dto::PayTaxRequest,
            crate::modules::finance::tax::dto::UpdateTaxSettingsRequest,
        )
    ),
    tags(
        (name = "System / Health", description = "System monitoring & database ping probe"),
        (name = "Authentication & Identity", description = "User registration & credential-based JWT issuance"),
        (name = "User Profile", description = "Authenticated user profile & identity"),
        (name = "User Management", description = "User CRUD & role assignments"),
        (name = "Access Control / RBAC", description = "Role-based access control & permission matrix"),
        (name = "Navigation / System Modules", description = "Dynamic sidebar groups & nested navigation modules"),
        (name = "Master Data / Plants", description = "Plant varieties and agronomic parameters"),
        (name = "Master Data / Locations", description = "Greenhouse branch locations and aggregated hole capacities"),
        (name = "Master Data / Installations", description = "Physical growing installation units and capacity mapping"),
        (name = "Master Data / Item Categories", description = "Operational item categories and aggregated count"),
        (name = "Master Data / Items", description = "Operational inventory catalog and precision threshold"),
        (name = "Master Data / Suppliers", description = "Vendor and supplier directory"),
        (name = "Master Data / Customers", description = "Customer and client directory"),
        (name = "Inventory / Warehouse Ledger", description = "Stock cards, warehouse movements, and physical stock opname"),
        (name = "Farm Operations / Planting Sessions", description = "Planting cycle management and target snapshot"),
        (name = "Farm Operations / Daily Readings", description = "Daily sensor logs, agronomy diagnostics, and compliance monitoring"),
        (name = "Plan / SOP", description = "Standard Operating Procedures repository & ordered steps"),
        (name = "Plan / Tasks", description = "Agile 6-stage Kanban board & farm tasks management"),
        (name = "Finance / Purchases", description = "Purchase headers, line items allocation, & inventory synchronization"),
        (name = "Finance / Sales", description = "Harvest sales pipeline, receivable management, & shipping subsidies"),
        (name = "Finance / HPP", description = "Cost of Goods Sold (HPP) engine, hole-ratio allocation, & unit economics"),
        (name = "Finance / Tax", description = "MSME Final Tax (PPh Final UMKM 0.5%), deadline tracking, & payment records")
    ),
    modifiers(&SecurityAddon),
    info(
        title = "AgriTech API",
        version = "1.0.0",
        description = "High-Performance Modular Agriculture REST API built with Rust, Axum, and SQLx."
    )
)]
pub struct ApiDoc;

pub fn mount_swagger_ui<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let config = Arc::new(Config::from("/api-docs/openapi.json"));

    let serve_swagger = move |path: Option<axum::extract::Path<String>>| {
        let config = config.clone();
        async move {
            let tail = path.map(|p| p.0).unwrap_or_default();
            let is_index = tail.is_empty() || tail == "index.html";

            if is_index {
                if let Ok(Some(file)) = utoipa_swagger_ui::serve("index.html", config.clone()) {
                    if let Ok(html) = String::from_utf8(file.bytes.to_vec()) {
                        let custom_style =
                            format!("<style>\n{}\n</style>\n</head>", theme::EMERALD_DARK_CSS);
                        let modified = html.replace("</head>", &custom_style);
                        return (
                            StatusCode::OK,
                            [(header::CONTENT_TYPE, "text/html; charset=utf-8".to_string())],
                            modified.into_bytes(),
                        );
                    }
                }
            }

            match utoipa_swagger_ui::serve(&tail, config) {
                Ok(Some(file)) => (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, file.content_type.to_string())],
                    file.bytes.to_vec(),
                ),
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    [(header::CONTENT_TYPE, "text/plain".to_string())],
                    b"Not Found".to_vec(),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "text/plain".to_string())],
                    e.to_string().into_bytes(),
                ),
            }
        }
    };

    router
        .route(
            "/api-docs/openapi.json",
            axum::routing::get(|| async { axum::Json(ApiDoc::openapi()) }),
        )
        .route(
            "/swagger-ui/custom.css",
            axum::routing::get(|| async {
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8".to_string())],
                    theme::EMERALD_DARK_CSS.as_bytes().to_vec(),
                )
            }),
        )
        .route("/swagger-ui", axum::routing::get(serve_swagger.clone()))
        .route("/swagger-ui/", axum::routing::get(serve_swagger.clone()))
        .route("/swagger-ui/*path", axum::routing::get(serve_swagger))
}
