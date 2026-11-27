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
            get(handler::list_categories).post(handler::create_category),
        )
        .route(
            "/:id",
            get(handler::get_category)
                .put(handler::update_category)
                .delete(handler::delete_category),
        )
}
