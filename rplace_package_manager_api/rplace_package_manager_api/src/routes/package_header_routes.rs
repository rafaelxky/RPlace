use core::option::Option::{self, None};

use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get,post,delete,put},
};
use serde_json::json;
use axum::http::{HeaderMap, HeaderValue};

use crate::{models::{app_state::AppState, package_file::package_file::PackageFile, registry::package_registry::{PackageRegistry, PackageRegistryCreateDto}}, service::auth_service::can_access};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/package/{name}", get(get_package_initial_file_no_version))
        .route("/package/{name}/{version}", get(get_package_initial_file))
        .route("/package/fetch_file/{version_header_id}/{path}", get(get_package_file))
        .route("/package", post(register_new_package_header))
}

// /package/fetch_file/{version_header_id}/{path}
/* returns:
{
    "repo_id": i32,
    "version": string,
    "header_id": i32,
    "file_hash": string,
    "file_path": string,
    "code": string
}
*/
async fn get_package_file(
    State(state): State<AppState>,
    Path(version_header_id): Path<i32>,
    Path(path): Path<String>,
) -> (StatusCode,impl IntoResponse){
    let link = state.db_provider.get_link_by_package_version_id_and_file_path(version_header_id, path).await;
    let link = match link {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!(
                {
                    "message": "could not get file link", 
                    "err": &e.to_string()
                }
            )));
        },
    };

    let file = state.db_provider.get_package_file_by_hash(link.file_hash).await;
    let file: PackageFile = match file {
        Ok(f) => f,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!({
                "message": "could not fetch file",
                "err": &e.to_string()
            })));
        },
    };

    return (StatusCode::OK, Json(json!({
        "header_id": link.package_version_id,
        "path": link.file_path,
        "file_hash": file.file_hash,
        "code": file.code,
    })));
}

// packages/{name}
/* returns: 
{
    "repo_id": i32,
    "version": string,
    "header_id": i32,
    "file_hash": string,
    "file_path": string,
    "code": string
}
*/
// path will be rplace.toml because its the initial fetch
// version will be the latest because none was specified
async fn get_package_initial_file_no_version(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> (StatusCode,impl IntoResponse) {
    let registry = state.db_provider.get_registry_by_name(name.clone()).await;
    let registry: PackageRegistry = match registry {
        Ok(reg) => reg,
        Err(e) => {
            return (StatusCode::NOT_FOUND,Json(json!({"msg": format!("could not find registry with name {}", name), "err": &e.to_string()})));
        }
    };
    let version_header = state
        .db_provider
        .get_latest_package_version_header_by_package_id(registry.id).await;

    let version_header = match version_header {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!(
                {
                    "msg": "could not fetch package version header", 
                    "err": &e.to_string()
                }
            )));
        },
    };

    let link = state.db_provider.get_link_by_package_version_id_and_file_path(version_header.id, "rplace".to_string()).await;
    let link = match link {
        Ok(l) => l,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!(
                {
                    "msg": "could not fetch file link", 
                    "err": &e.to_string()
                }
            )));
        },
    };

    let file = state.db_provider.get_package_file_by_hash(link.file_hash).await;
    let file = match file {
        Ok(f) => f,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!(
                {
                    "message": "could not fetch file",
                    "err": &&e.to_string(),
                }
            )));
        },
    };

    return (StatusCode::OK,Json(json!(
        {
            "repo_id": registry.id,
            "version": version_header.version,
            "header_id": version_header.id,
            "file_hash": file.file_hash,
            "file_path": "rplace.toml",
            "code": file.code
        }
    )));
}
// packages/{name}/{version}
/* returns;
{
    "repo_id": i32,
    "version": strign,
    "header_id": i32,
    "file_hash": string,
    "file_path": string,
    "code": string
}
*/
// path will be rplace.toml because its the initial fetch
async fn get_package_initial_file(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Path(version): Path<String>,
) -> (StatusCode,impl IntoResponse) {
    let registry = state.db_provider.get_registry_by_name(name.clone()).await;
    let registry: PackageRegistry = match registry {
        Ok(reg) => reg,
        Err(e) => {
            return (StatusCode::NOT_FOUND,Json(json!({"msg": format!("could not find package with name {}",name), "err": &e.to_string()})));
        }
    };
    let version_header = state
        .db_provider
        .get_package_version_header_by_package_id_and_version(registry.id, version.clone()).await;

    let version_header = match version_header {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::NOT_FOUND,Json(json!(
                {
                    "msg": format!("could not find version {} for package {}", version, name), 
                    "err": &e.to_string()
                }
            )));
        },
    };

    let link = state.db_provider.get_link_by_package_version_id_and_file_path(version_header.id, "rplace".to_string()).await;
    let link = match link {
        Ok(l) => l,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!(
                {
                    "msg": "could not fetch file link", 
                    "err": &e.to_string()
                }
            )));},
    };

    let file = state.db_provider.get_package_file_by_hash(link.file_hash).await;
    let file = match file {
        Ok(f) => f,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR,Json(json!(
                {
                    "message": "could not fetch file",
                    "err": &&e.to_string(),
                }
            )));
        },
    };

    return (StatusCode::OK,Json(json!(
        {
            "repo_id": registry.id,
            "version": version_header.version,
            "header_id": version_header.id,
            "file_hash": file.file_hash,
            "file_path": "rplace.toml",
            "code": file.code
        }
    )));
}

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
            return (StatusCode::UNAUTHORIZED, Json(json!({
                "message": "auth header not found",
            })));
        }
    };

    let tok = tok.to_str();
    let tok = match tok {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "could not parse auth header to string",
                "err": &e.to_string()
            })));
        }
    };

    let claim = can_access(tok);
    let claim = match claim {
        Ok(c) => c,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, Json(json!({
                "message": "invalid jwt token",
                "err": e.to_string()
            })));
        }
    };

    let user = state.db_provider.get_user_by_id(claim.user_id).await;

    let user = match user {
        Ok(u) => u,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, Json(json!({
                "message": "invalid jwt token",
                "err": e.to_string()
            })));
        }
    };

    let res = state.db_provider.new_registry(new_package, user.id).await;

    let res = match res {
        Ok(res) => {
            res
        },
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "could not create new registry",
                "err": &err.to_string()
            })));
        },
    };

    return (StatusCode::OK, Json(json!({
        "id": &res.id,
        "name": &res.name,
        "created_at": &res.created_at,
        "creator_id": &res.creator_id
    })));
}

