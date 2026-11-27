pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::list_sops).post(handler::create_sop))
        .route(
            "/:id",
            get(handler::get_sop)
                .put(handler::update_sop)
                .delete(handler::delete_sop),
        )
}
