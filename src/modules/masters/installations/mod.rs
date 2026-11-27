pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handler::list_installations).post(handler::create_installation),
        )
        .route(
            "/:id",
            get(handler::get_installation)
                .put(handler::update_installation)
                .delete(handler::delete_installation),
        )
}
