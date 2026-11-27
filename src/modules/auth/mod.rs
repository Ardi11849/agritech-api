pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::post;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handler::register))
        .route("/login", post(handler::login))
}
