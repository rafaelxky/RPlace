use core::option::Option::{self, None};

use axum::http::{HeaderMap, HeaderValue};
use axum::{
    Json, Router,
    extract::{State},
    http::StatusCode,
    response::IntoResponse,
    routing::{post},
};
use serde_json::json;

use crate::{
    models::{
        app_state::AppState,
        registry::package_registry::{PackageRegistryCreateDto},
    },
    service::auth_service::can_access,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/package", post(register_new_package_header))
}

//  /package POST
// must be logged in
// jwt token in header
// body:
/*
{
    "name": string
}

returns:
{
    "id": i32,
    "name": string,
    "created_at": timedate,
    "creator_id": i32
}
*/
pub async fn register_new_package_header(
    State(state): State<AppState>,
    header: HeaderMap,
    Json(package): Json<PackageRegistryCreateDto>,
) -> (StatusCode, impl IntoResponse) {
    let new_package = package;
    let tok: Option<&HeaderValue> = header.get("Authorization");
    let tok: &HeaderValue = match tok {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "auth header not found",
                })),
            );
        }
    };

    let tok = tok.to_str();
    let tok = match tok {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not parse auth header to string",
                    "err": &e.to_string()
                })),
            );
        }
    };

    let claim = can_access(tok);
    let claim = match claim {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "invalid jwt token, cannot access",
                    "err": e.to_string()
                })),
            );
        }
    };

    let user = state.db_provider.get_user_by_id(claim.user_id).await;

    let user = match user {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "invalid jwt token no such user",
                    "err": e.to_string()
                })),
            );
        }
    };

    let res = state.db_provider.new_registry(new_package, user.id).await;

    let res = match res {
        Ok(res) => res,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not create new registry",
                    "err": &err.to_string()
                })),
            );
        }
    };

    return (
        StatusCode::OK,
        Json(json!({
            "id": &res.id,
            "name": &res.package_name,
            "created_at": &res.created_at,
            "creator_id": &res.creator_id
        })),
    );
}
