use std::{collections::HashMap, sync::Arc};

use anyhow::{Ok, Result};
use axum::{
    Router,
    http::{Request, StatusCode},
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use rplace_package_manager_api::{app::app, db::sqlite_db::SqliteDb};
use tower::ServiceExt;

const USERNAME: &str = "user123";
const EMAIL: &str = "usermail@gmail.com";
const PASSWORD: &str = "password123";
const PACKAGE_NAME: &str = "my_package123";
const VERSION: &str = "1.0.0.0";
const CODE: &str = "hello world";
const PATH: &str = "src/main.rs";

const JSON_HEADER_KEY: &str = "Content-Type";
const JSON_HEADER_VALUES: &str = "application/json";
const AUTH_HEADER_KEY: &str = "Authorization";

const CREATE_USER_URI: &str = "/user";
const LOGGIN_URI: &str = "/loggin";
const CREATE_PACKAGE_URI: &str = "/package";
const CREATE_VERSION_URI: &str = "/package/version";
const CREATE_FILE_URI: &str = "/file";

async fn setup(db_name: &str) -> Result<(Arc<SqliteDb>, Router)> {
    dotenv().ok();
    let _ = std::fs::remove_file(db_name);
    let db = SqliteDb::new_with_db_url(db_name).await?;
    db.migrate().await?;
    let db: Arc<SqliteDb> = Arc::new(db);
    let app = app(db.clone()).await?;
    Ok((db, app))
}

#[tokio::test]
async fn create_full() -> Result<()> {
    let (_db, app) = setup("db/create_full.db").await?;

    // create user
    let create_user = serde_json::json!({
        "name": USERNAME,
        "email": EMAIL,
        "password": PASSWORD
    })
    .to_string();

    let request = Request::builder()
        .uri(CREATE_USER_URI)
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .method("POST")
        .body(create_user)?;

    let created_user = app.clone().oneshot(request).await?;
    println!("created user status: {}", created_user.status());

    let bytes = created_user.into_body().collect().await?.to_bytes();
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;

    let id = body.get("id").unwrap().as_i64().unwrap();
    let name = body.get("name").unwrap().as_str().unwrap();
    // todo: checks response

    // get token/ loggin
    let request = serde_json::json!({
        "email": EMAIL,
        "password": PASSWORD,
    })
    .to_string();

    let request = Request::builder()
        .uri(LOGGIN_URI)
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .method("POST")
        .body(request)?;

    let token = app.clone().oneshot(request).await?;
    let status = token.status();
    let token = token.into_body();
    let bytes = token.collect().await?.to_bytes();

    println!("token status: {}", status);
    if status != StatusCode::OK {
        println!("token body: {}", String::from_utf8_lossy(&bytes));
    }

    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;
    let tok = body
        .get("token")
        .expect("did not return JWT token")
        .as_str()
        .unwrap();

    // todo: check token

    // create new package
    let create_package = serde_json::json!({
        "name": PACKAGE_NAME
    })
    .to_string();

    let request = Request::builder()
        .uri(CREATE_PACKAGE_URI)
        .method("POST")
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .header(AUTH_HEADER_KEY, format!("Bearer {tok}"))
        .body(create_package)?;

    let created_package = app.clone().oneshot(request).await?;
    let status = created_package.status();
    let bytes = created_package.into_body().collect().await?.to_bytes();
    println!("create package status: {}", status);
    if status != StatusCode::OK {
        println!("created package body: {}", String::from_utf8_lossy(&bytes));
    }
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;

    let package_id = body.get("id").unwrap().as_i64().unwrap();
    let package_name = body.get("name").unwrap().as_str().unwrap();
    let created_at = body.get("created_at").unwrap().as_str().unwrap();
    let creator_id = body.get("creator_id").unwrap().as_i64().unwrap();

    // todo: check created package

    // create new version
    let create_version = serde_json::json!({
        "package_name": PACKAGE_NAME,
        "version": VERSION,
    })
    .to_string();

    let request = Request::builder()
        .uri(CREATE_VERSION_URI)
        .method("POST")
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .header(AUTH_HEADER_KEY, format!("Bearer {tok}"))
        .body(create_version)?;

    let created_version = app.clone().oneshot(request).await?;
    let status = created_version.status();
    println!("create version status: {}", created_version.status());
    let bytes = created_version.into_body().collect().await?.to_bytes();
    if status != StatusCode::OK {
        println!("created version body: {}", String::from_utf8_lossy(&bytes));
    }
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;

    let version_id = body.get("id").unwrap().as_i64().unwrap();
    let version_name = body.get("version").unwrap().as_str().unwrap();
    let created_at = body.get("created_at").unwrap().as_str().unwrap();
    let version_package_id = body.get("package_id").unwrap().as_i64().unwrap();

    // todo: check created version

    // todo: get registry and version header id by name

    // insert file
    let create_file = serde_json::json!({
        "registry_id": package_id,
        "version_header_id": version_id,
        "code": CODE,
        "path": PATH
    })
    .to_string();

    let request = Request::builder()
        .uri(CREATE_FILE_URI)
        .method("POST")
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .header(AUTH_HEADER_KEY, format!("Bearer {tok}"))
        .body(create_file)?;

    let created_file = app.clone().oneshot(request).await?;
    let status = created_file.status();
    println!("create file status: {}", created_file.status());

    let bytes = created_file.into_body().collect().await?.to_bytes();

    if status != StatusCode::OK {
        println!("create file body: {}", String::from_utf8_lossy(&bytes));
    }
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;

    let path = body.get("path").unwrap().as_str().unwrap();
    let file_hash = body.get("file_hash").unwrap().as_str().unwrap();

    // todo: test file

    Ok(())
}
