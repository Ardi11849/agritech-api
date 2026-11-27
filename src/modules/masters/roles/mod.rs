pub mod dto;
pub mod handler;
pub mod repository;
pub mod service;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::get_roles).post(handler::create_role))
        .route(
            "/:id",
            get(handler::get_role)
                .put(handler::update_role)
                .delete(handler::delete_role),
        )
        .route(
            "/rolePermissions",
            get(handler::get_all_permissions).post(handler::create_permission),
        )
        .route(
            "/rolePermissions/:id",
            get(handler::get_permission_by_id)
                .put(handler::update_permission)
                .delete(handler::delete_permission),
        )
        .route(
            "/rolePermissions/role/:id_role",
            get(handler::get_permissions_by_role),
        )
}
