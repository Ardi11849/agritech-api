pub mod ai_audit;
pub mod audit;
pub mod summary;

use axum::Router;

use crate::AppState;

/// Pilar 5: Reports (/api/v1/reports/*)
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/summary", summary::router())
        .nest("/audit", audit::router())
        .nest("/ai-audit", ai_audit::router())
}
