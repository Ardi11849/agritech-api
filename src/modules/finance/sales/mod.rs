pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::get_sales).post(handler::create_sale))
        .route(
            "/:id",
            get(handler::get_sale)
                .put(handler::update_sale)
                .delete(handler::delete_sale),
        )
}
