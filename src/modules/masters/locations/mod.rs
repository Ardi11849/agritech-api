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
            get(handler::list_locations).post(handler::create_location),
        )
        .route(
            "/:id",
            get(handler::get_location)
                .put(handler::update_location)
                .delete(handler::delete_location),
        )
}
