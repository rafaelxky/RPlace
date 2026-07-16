use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    models::{
        app_state::AppState,
        link::link::LinkCreateDto,
        package_file::package_file::{PackageFile, PackageFileCreateDto},
    },
    service::auth_service::can_access,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/file", post(new_file))
}

// creates a new file for that version
// requires loggin
// JWT token in header
// /file POST
/*
body:
{
    "registry_id": i32,
    "version_header_id": i32,
    "code": string,
    "path": string
}

returns:
{
    "path": string,
    "file_hash": string
}
 */
pub async fn new_file(
    State(state): State<AppState>,
    header: HeaderMap,
    Json(file_request): Json<PackageFileCreateDto>,
) -> (StatusCode, impl IntoResponse) {
    let file_request: PackageFileCreateDto = file_request;
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not parse auth header to string",
                    "err": &e.to_string()
                })),
            );
        }
    };

    let res = can_access(tok);
    let tok = match res {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "invalid JWT token"
                })),
            );
        }
    };

    let registry = state
        .db_provider
        .get_registry_by_id(file_request.registry_id)
        .await;
    let registry = match registry {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!(
                    {
                        "message": "no registry found with provided id",
                        "err": &e.to_string()
                    }
                )),
            );
        }
    };

    if registry.creator_id != tok.user_id {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                 "message": "this user doesn't own this package"
            })),
        );
    }

    let header = state
        .db_provider
        .get_package_version_header_by_id(file_request.version_header_id)
        .await;
    let header = match header {
        Ok(h) => h,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!(
                    {
                        "message": "header not found for provided ID",
                        "err": &err.to_string()
                    }
                )),
            );
        }
    };

    if header.package_id != registry.id {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!(
                {
                    "message": "header id and version package_id missmatch, the version header does not belong to this package",
                }
            )),
        );
    }

    let mut hasher = Sha256::new();
    hasher.update(file_request.code.clone());
    let hash = hex::encode(hasher.finalize());

    let maybe_file = state
        .db_provider
        .get_package_file_by_hash(hash.clone())
        .await;
    let file = match maybe_file {
        Ok(Some(f)) => {
            // found file, return and create new link
            f
        }
        Ok(None) => {
            // didnt find file, upload and create new link
            let f = PackageFile {
                code: file_request.code,
                file_hash: hash,
            };
            let f = state.db_provider.new_file(f).await;
            let f = match f {
                Ok(f) => f,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "message": "could not create new file",
                            "err": &e.to_string()
                        })),
                    );
                }
            };
            f
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

    let link = LinkCreateDto {
        file_path: file_request.path,
        package_version_id: file_request.version_header_id,
        file_hash: file.file_hash.clone(),
    };
    let link = state.db_provider.new_link(link).await;

    let link = match link {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not create file link",
                    "err": &e.to_string()
                })),
            );
        }
    };

    return (
        StatusCode::OK,
        Json(json!({
            "path": &link.file_path,
            "file_hash": &link.file_hash
        })),
    );
}
