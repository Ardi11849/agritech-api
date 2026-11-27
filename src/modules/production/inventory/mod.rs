pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/summary", get(handler::get_summary))
        .route("/movements", post(handler::record_movement))
        .route("/stock-opname", post(handler::record_stock_opname))
        .route("/items/:id/stock-card", get(handler::get_stock_card))
}
