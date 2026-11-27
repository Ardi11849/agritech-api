pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::get_logs))
        .route("/modules", get(handler::get_modules))
}
