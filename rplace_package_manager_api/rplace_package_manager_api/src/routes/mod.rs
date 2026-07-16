use axum::Router;

use crate::models::app_state::AppState;

pub mod package_header_routes;
pub mod package_version_routes;
pub mod user_routes;
pub mod auth;
pub mod package_file_routes;

pub fn router() -> Router<AppState> {
    Router::new()
    .merge(package_header_routes::routes())
    .merge(user_routes::routes())
    .merge(auth::routes())
    .merge(package_version_routes::routes())
    .merge(package_file_routes::routes())
}