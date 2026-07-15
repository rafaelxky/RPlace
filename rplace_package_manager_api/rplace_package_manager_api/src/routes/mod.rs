use axum::Router;

use crate::models::app_state::AppState;

pub mod package_routes;

pub fn router() -> Router<AppState> {
    Router::new().merge(package_routes::routes())
}