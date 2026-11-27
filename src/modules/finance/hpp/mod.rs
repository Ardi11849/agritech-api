pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::get_hpp_list))
        .route("/:session_id", get(handler::get_session_rhpp))
        .route("/:session_id/lock-snapshot", post(handler::lock_snapshot))
}
