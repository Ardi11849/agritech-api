pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::{get, patch};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::list_tasks).post(handler::create_task))
        .route(
            "/:id",
            get(handler::get_task)
                .put(handler::update_task)
                .delete(handler::delete_task),
        )
        .route("/:id/status", patch(handler::update_task_status))
}
