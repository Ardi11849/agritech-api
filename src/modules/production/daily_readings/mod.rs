pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::{delete, get};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/unreported-today", get(handler::get_unreported_today))
        .route("/:id", delete(handler::delete_daily_reading))
}
