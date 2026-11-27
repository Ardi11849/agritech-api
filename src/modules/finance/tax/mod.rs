pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(handler::get_tax_dashboard))
        .route("/pay", post(handler::pay_tax))
        .route("/settings", put(handler::update_tax_settings))
        .route("/reports/:id", delete(handler::delete_tax_report))
}
