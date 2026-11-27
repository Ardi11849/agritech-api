pub mod auth;
pub mod finance;
pub mod health;
pub mod masters;
pub mod plan;
pub mod production;
pub mod reports;

// Re-ekspor untuk kompatibilitas internal crate
pub use masters::{roles as role, system_modules, users as user};
pub use production::{daily_readings, inventory, sessions};

use axum::Router;

use crate::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        // Core probes & auth
        .nest("/api/v1", health::router())
        .nest("/api/v1/auth", auth::router())
        // 5 Pilar Utama (Kanonik)
        .nest("/api/v1/masters", masters::router())
        .nest("/api/v1/plan", plan::router())
        .nest("/api/v1/production", production::router())
        .nest("/api/v1/finance", finance::router())
        .nest("/api/v1/reports", reports::router())
        // Alias Kompatibilitas Mundur (Backward Compatibility)
        .nest("/api/v1/users", masters::users::router())
        .nest("/api/v1/roles", masters::roles::router())
        .nest("/api/v1/system-modules", masters::system_modules::router())
        .nest("/api/v1/inventory", production::inventory::router())
        .nest("/api/v1/sessions", production::sessions::router())
        .nest(
            "/api/v1/daily-readings",
            production::daily_readings::router(),
        )
        .nest("/api/v1/purchases", finance::purchases::router())
        .nest("/api/v1/sales", finance::sales::router())
        .nest("/api/v1/hpp", finance::hpp::router())
        .nest("/api/v1/tax", finance::tax::router())
        .nest("/api/v1/laporan", reports::summary::router())
        .nest("/api/v1/audit", reports::audit::router())
        .nest("/api/v1/ai-audit", reports::ai_audit::router())
}
