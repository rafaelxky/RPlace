use core::option::Option::{self, None};

use axum::http::{HeaderMap, HeaderValue};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
};
use serde_json::json;

use crate::{
    models::{
        app_state::AppState,
        package_file::package_file::PackageFile,
        registry::package_registry::{PackageRegistry},
    },
};

// all routes in this file bellong to the fetching pipeline
// they are used to get the file data from the server to the client
// none of these should be secured since they are all for getting public data
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/package/fetch_file/{version_header_id}/{path}",
            get(get_package_file),
        )
        .route("/package/{name}", get(get_package_initial_file_no_version))
        .route("/package/{name}/{version}", get(get_package_initial_file))
}

// 1st step for fetching
// packages/{name} GET
// returns the rplace.toml file content and other relevant content to get the files
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
) -> (StatusCode, impl IntoResponse) {
    let registry = state.db_provider.get_registry_by_name(name.clone()).await;
    let registry: PackageRegistry = match registry {
        Ok(reg) => reg,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(
                    json!({"msg": format!("could not find registry with name {}", name), "err": &e.to_string()}),
                ),
            );
        }
    };
    let version_header = state
        .db_provider
        .get_latest_package_version_header_by_package_id(registry.id)
        .await;

    let version_header = match version_header {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "msg": "could not fetch package version header",
                        "err": &e.to_string()
                    }
                )),
            );
        }
    };

    let link = state
        .db_provider
        .get_link_by_package_version_id_and_file_path(version_header.id, "rplace".to_string())
        .await;
    let link = match link {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "msg": "could not fetch file link",
                        "err": &e.to_string()
                    }
                )),
            );
        }
    };

    let file = state
        .db_provider
        .get_package_file_by_hash(link.file_hash)
        .await;
    let file = match file {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "message": "could not find file",
                    }
                )),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "message": "could not fetch file",
                        "err": &&e.to_string(),
                    }
                )),
            );
        }
    };

    return (
        StatusCode::OK,
        Json(json!(
            {
                "repo_id": registry.id,
                "version": version_header.version,
                "header_id": version_header.id,
                "file_hash": file.file_hash,
                "file_path": "rplace.toml",
                "code": file.code
            }
        )),
    );
}

// 1st step for fetching
// packages/{name}/{version} GET
// returns the rplace.toml file content and other relevant content to get the files
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
) -> (StatusCode, impl IntoResponse) {
    let registry = state.db_provider.get_registry_by_name(name.clone()).await;
    let registry: PackageRegistry = match registry {
        Ok(reg) => reg,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(
                    json!({"msg": format!("could not find package with name {}",name), "err": &e.to_string()}),
                ),
            );
        }
    };
    let version_header = state
        .db_provider
        .get_package_version_header_by_package_id_and_version(registry.id, version.clone())
        .await;

    let version_header = match version_header {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!(
                    {
                        "msg": format!("could not find version {} for package {}", version, name),
                        "err": &e.to_string()
                    }
                )),
            );
        }
    };

    let link = state
        .db_provider
        .get_link_by_package_version_id_and_file_path(version_header.id, "rplace".to_string())
        .await;
    let link = match link {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "msg": "could not fetch file link",
                        "err": &e.to_string()
                    }
                )),
            );
        }
    };

    let file = state
        .db_provider
        .get_package_file_by_hash(link.file_hash)
        .await;
    let file = match file {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "message": "file not found",
                    }
                )),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "message": "could not fetch file",
                        "err": &e.to_string(),
                    }
                )),
            );
        }
    };

    return (
        StatusCode::OK,
        Json(json!(
            {
                "repo_id": registry.id,
                "version": version_header.version,
                "header_id": version_header.id,
                "file_hash": file.file_hash,
                "file_path": "rplace.toml",
                "code": file.code
            }
        )),
    );
}

// 2nd step for fetching
// after you get the header data and rplace.toml
// this is used to fetch the rest of the files using that data
// gets a file from a specific package and version
// /package/fetch_file/{version_header_id}/{path} GET
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
) -> (StatusCode, impl IntoResponse) {
    let link = state
        .db_provider
        .get_link_by_package_version_id_and_file_path(version_header_id, path)
        .await;
    let link = match link {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                    {
                        "message": "could not get file link",
                        "err": &e.to_string()
                    }
                )),
            );
        }
    };

    let file = state
        .db_provider
        .get_package_file_by_hash(link.file_hash)
        .await;
    let file: PackageFile = match file {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not find file",
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not fetch file",
                    "err": &e.to_string()
                })),
            );
        }
    };

    return (
        StatusCode::OK,
        Json(json!({
            "header_id": link.package_version_id,
            "path": link.file_path,
            "file_hash": file.file_hash,
            "code": file.code,
        })),
    );
}
