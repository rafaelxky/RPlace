use crate::{models::{
    app_state::AppState,
    package_version_header::package_version_header::PackageVersionHeaderCreateDto,
}, service::auth_service::can_access};
use axum::{
    Json, Router, extract::State, http::{HeaderMap, HeaderValue, StatusCode}, response::IntoResponse, routing::*,
};
use serde_json::json;

pub fn routes() -> Router<AppState> {
    Router::new()
}

// creates new package version for package
// must be logged in 
// jwt token in header
/*
body:
{
    "package_name": string,
    "version": i32
}

returns:
{
    "id": i32,
    "version": string,
    "created_at": datetime,
    "package_id": i32
}
*/
async fn new_package_version(
    State(state): State<AppState>,
    header: HeaderMap,
    Json(package): Json<PackageVersionHeaderCreateDto>,
) -> (StatusCode, impl IntoResponse) {
    let new_version = package;
    let tok: Option<&HeaderValue> = header.get("Authorization");
    let tok = match tok {
        Some(t) => t.to_str(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "auth header not found",
                })),
            );
        }
    };
    let tok = match tok {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "could not parse jwt token to string",
                "err": &e.to_string()
            })));
        } 
    };

    let package = state.db_provider.get_registry_by_name(new_version.package_name.clone()).await;
    let package = match package {
        Ok(p) => p,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!(
                {
                    "message": format!{"package with name {} not found", new_version.package_name}
                }
            )));
        },
    };

    let res = can_access(tok);
    let tok = match res {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, Json(json!(
                {
                    "message": "invalid jwt token",
                    "err": &e.to_string()
                }
            )));
        },
    };

    if tok.user_id != package.creator_id {
        return (StatusCode::UNAUTHORIZED, Json(json!(
            {
                "message": "this user cannot create a new version for this package"
            }
        )));
    }

    let res = state.db_provider.new_package_version(new_version.version, package.id).await;
    let res = match res {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!(
                {
                    "message": "could not create new version",
                    "err": &e.to_string()
                }
            )));
        },
    };

    return (StatusCode::OK, Json(json!({
        "id": &res.id,
        "version": &res.version,
        "created_at": &res.created_at,
        "package_id": &res.package_id
    })));
}
