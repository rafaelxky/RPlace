use axum::Router;

use crate::models::app_state::AppState;

pub mod package_routes;
pub mod user_routes;
pub mod auth;

pub fn router() -> Router<AppState> {
    Router::new()
    .merge(package_routes::routes())
    .merge(user_routes::routes())
    .merge(auth::routes())
}