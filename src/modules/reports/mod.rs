use axum::Router;

use crate::AppState;

/// Pilar 5: Reports (/api/v1/reports/*)
/// Kerangka router untuk sub-modul masa depan: summary, audit, ai_audit
pub fn router() -> Router<AppState> {
    Router::new()
}
