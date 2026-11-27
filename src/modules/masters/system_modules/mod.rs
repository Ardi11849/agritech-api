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
            "/groups",
            get(handler::get_groups).post(handler::create_group),
        )
        .route(
            "/groups/:id",
            get(handler::get_group_by_id)
                .put(handler::update_group)
                .delete(handler::delete_group),
        )
        .route(
            "/modules",
            get(handler::get_modules).post(handler::create_module),
        )
        .route(
            "/modules/:id",
            get(handler::get_module_by_id)
                .put(handler::update_module)
                .delete(handler::delete_module),
        )
}
