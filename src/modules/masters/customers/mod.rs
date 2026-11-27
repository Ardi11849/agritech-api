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
            get(handler::list_customers).post(handler::create_customer),
        )
        .route(
            "/:id",
            get(handler::get_customer)
                .put(handler::update_customer)
                .delete(handler::delete_customer),
        )
}
