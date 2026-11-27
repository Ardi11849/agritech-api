pub mod hpp;
pub mod purchases;
pub mod sales;
pub mod tax;

use axum::Router;

use crate::AppState;

/// Pilar 4: Finance (/api/v1/finance/*)
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/purchases", purchases::router())
        .nest("/sales", sales::router())
        .nest("/hpp", hpp::router())
        .nest("/tax", tax::router())
}
