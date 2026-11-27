pub mod customers;
pub mod installations;
pub mod item_categories;
pub mod items;
pub mod locations;
pub mod plants;
pub mod roles;
pub mod suppliers;
pub mod system_modules;
pub mod users;

pub use roles as role;
pub use users as user;

use axum::Router;

use crate::AppState;

/// Pilar 1: Masters (/api/v1/masters/*)
/// Data Induk Tanaman, Lokasi, Instalasi, Barang, Mitra, Pengguna, Hak Akses, dan Modul Sistem.
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/plants", plants::router())
        .nest("/locations", locations::router())
        .nest("/installations", installations::router())
        .nest("/item-categories", item_categories::router())
        .nest("/items", items::router())
        .nest("/suppliers", suppliers::router())
        .nest("/customers", customers::router())
        .nest("/users", users::router())
        .nest("/roles", roles::router())
        .nest("/system-modules", system_modules::router())
}
