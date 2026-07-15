use crate::models::app_state::AppState;
use axum::{Router, extract::State, Json, response::IntoResponse, http::StatusCode, routing::*};
use serde_json::json;

pub fn routes() -> Router<AppState>{
    Router::new()
}

async fn new_package_version(State(state): State<AppState>, Json(package): Json<()>) -> (StatusCode, impl IntoResponse){
    return (StatusCode::OK, Json(json!({})));
}