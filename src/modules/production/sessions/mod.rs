pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(handler::list_sessions).post(handler::create_session),
        )
        .route(
            "/:id",
            get(handler::get_session)
                .put(handler::update_session)
                .delete(handler::delete_session),
        )
        .route("/:id/complete-harvest", post(handler::complete_harvest))
        .route(
            "/:session_id/daily-readings",
            get(crate::modules::daily_readings::handler::list_session_daily_readings)
                .post(crate::modules::daily_readings::handler::create_session_daily_reading),
        )
}
