pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::list_items).post(handler::create_item))
        .route(
            "/:id",
            get(handler::get_item)
                .put(handler::update_item)
                .delete(handler::delete_item),
        )
}
