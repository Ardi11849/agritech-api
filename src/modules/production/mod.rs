pub mod daily_readings;
pub mod inventory;
pub mod sessions;

use axum::Router;

use crate::AppState;

/// Pilar 3: Production (/api/v1/production/*)
/// Mencakup siklus sesi tanam, monitoring sensor/catatan harian, dan gudang & inventori.
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/sessions", sessions::router())
        .nest("/daily-readings", daily_readings::router())
        .nest("/inventory", inventory::router())
}
