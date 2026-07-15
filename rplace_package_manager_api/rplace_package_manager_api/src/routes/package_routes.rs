use axum::{
    Json, Router, body::Body, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::get,
};
use serde_json::json;

use crate::models::{app_state::AppState, package_file::package_file::PackageFile, registry::package_registry::PackageRegistry};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/package/{name}", get(get_package_initial_file_no_version))
        .route("/package/{name}/{version}", get(get_package_initial_file))
        .route("/package/fetch_file/{version_header_id}/{path}", get(get_package_file))
}

async fn get_package_file(
    State(state): State<AppState>,
    Path(version_header_id): Path<i32>,
    Path(path): Path<String>,
) -> impl IntoResponse{
    let link = state.db_provider.get_link_by_package_version_id_and_file_path(version_header_id, path).await;
    let link = match link {
        Ok(h) => h,
        Err(e) => {
            return Json(json!(
                {
                    "message": "could not get file link", 
                    "err": &e.to_string()
                }
            ));
        },
    };

    let file = state.db_provider.get_package_file_by_hash(link.file_hash).await;
    let file: PackageFile = match file {
        Ok(f) => f,
        Err(e) => {
            return Json(json!({
                "message": "could not fetch file",
                "err": &e.to_string()
            }));
        },
    };

    return Json(json!({
        "header_id": link.package_version_id,
        "path": link.file_path,
        "file_hash": file.file_hash,
        "code": file.code,
    }));
}

// packages/{name}
async fn get_package_initial_file_no_version(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
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

pub async fn register_new_package(State(state): State<AppState>, Json(package): Json<PackageRegistry>) -> (StatusCode, impl IntoResponse) {
    let new_package = package;
    return (StatusCode::OK, Json(json!({})));
}