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
            get(handler::list_suppliers).post(handler::create_supplier),
        )
        .route(
            "/:id",
            get(handler::get_supplier)
                .put(handler::update_supplier)
                .delete(handler::delete_supplier),
        )
}
