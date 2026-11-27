pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::list_plants).post(handler::create_plant))
        .route(
            "/:id",
            get(handler::get_plant)
                .put(handler::update_plant)
                .delete(handler::delete_plant),
        )
}
