pub mod sop;
pub mod tasks;

use axum::Router;

use crate::AppState;

/// Pilar 2: Plan (/api/v1/plan/*)
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/sops", sop::router())
        .nest("/tasks", tasks::router())
}
