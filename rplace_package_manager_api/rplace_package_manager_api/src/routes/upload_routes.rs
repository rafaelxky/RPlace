use axum::Router;

use crate::models::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}

