pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::get_users).post(handler::create_user))
        .route("/profile", get(handler::get_profile))
        .route(
            "/:id",
            get(handler::get_user)
                .put(handler::update_user)
                .delete(handler::delete_user),
        )
}
