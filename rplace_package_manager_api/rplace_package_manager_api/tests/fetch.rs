use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use axum::{
    Router, body::Body, http::{Request, StatusCode},
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use rplace_package_manager_api::{
    app::app,
    db::sqlite_db::SqliteDb,
};
use std::result::Result::Ok;
use tower::ServiceExt;

const PACKAGE_NAME: &str = "my_package";
const WRONG_PACKAGE_NAME: &str = "not_my_package";
const VERSION: &str = "1.0.0.0";
const VERSION_ID: i32 = 1;
const FILE_PATH: &str = "rplace.toml";
const CODE: &str = "hello world";
const USER_ID: i32 = 1;
const PACKAGE_ID: i32 = 1;
const FILE_HASH: &str = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

const INSERT_USER: &str = "INSERT INTO users (name,email,password_hash) VALUES (?,?,?);";
const INSERT_REGISTRY: &str =
    "INSERT INTO package_registry (package_name,creator_id) VALUES (?,?);";
const INSERT_VERSION: &str =
    "INSERT INTO package_version_header (version, package_id) VALUES (?,?);";
const INSERT_LINK: &str =
    "INSERT INTO links (file_path, file_hash,package_version_id) VALUES (?,?,?);";
const INSERT_PACKAGE_FILE: &str = "INSERT INTO package_file (file_hash,code) VALUES (?,?);";
const USERNAME: &str = "usr";
const EMAIL: &str = "example@gmail.com";
const PASSWORD_HASH: &str = "password123";

async fn setup(db_name: &str) -> Result<(Arc<SqliteDb>, Router)>{
    let _ = std::fs::remove_file(db_name);
    let db = SqliteDb::new_with_db_url(db_name).await?;
    db.migrate().await?;
    let db: Arc<SqliteDb> = Arc::new(db);
    let app = app(db.clone()).await?;
    // insert user
    sqlx::query(INSERT_USER)
        .bind(USERNAME)
        .bind(EMAIL)
        .bind(PASSWORD_HASH)
        .execute(&db.pool)
        .await?;
    // insert package
    sqlx::query(INSERT_REGISTRY)
        .bind(PACKAGE_NAME)
        .bind(USER_ID)
        .execute(&db.pool)
        .await?;
    // insert version
    sqlx::query(INSERT_VERSION)
        .bind(VERSION)
        .bind(PACKAGE_ID)
        .execute(&db.pool)
        .await?;
    // insert package_file
    sqlx::query(INSERT_PACKAGE_FILE)
        .bind(FILE_HASH)
        .bind(CODE)
        .execute(&db.pool)
        .await?;
    // insert link
    sqlx::query(INSERT_LINK)
        .bind(FILE_PATH)
        .bind(FILE_HASH)
        .bind(VERSION_ID)
        .execute(&db.pool)
        .await?;

    Ok((db, app))

}

#[tokio::test]
async fn get_package_initial_file_no_version_success() -> Result<()> {
    dotenv().ok();
    const DB_NAME: &str = "db/test_initial_no_version.db";
    let (_db,app) = setup(DB_NAME).await?;

    let request = Request::builder()
        .uri(format!("/package/{}", PACKAGE_NAME))
        .method("GET")
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();

    let bytes = response.into_body().collect().await?.to_bytes();
    if status != StatusCode::OK {
        println!("status: {}", status);
        println!("body: {:?}", String::from_utf8_lossy(&bytes));
        panic!("wrong status code");
    }
    assert_eq!(status, StatusCode::OK);
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;
    let repo_id = body.get("repo_id").unwrap();
    let version = body.get("version").unwrap();
    let file_hash = body.get("file_hash").unwrap();
    let file_path = body.get("file_path").unwrap();
    let code = body.get("code").unwrap();
    assert_eq!(repo_id, 1);
    assert_eq!(version, VERSION);
    assert_eq!(file_hash, FILE_HASH);
    assert_eq!(file_path, FILE_PATH);
    assert_eq!(code, CODE);

    Ok(())
}

#[tokio::test]
async fn get_package_initial_file_no_version_fail() -> Result<()> {
    dotenv().ok();
    const DB_NAME: &str = "db/test_initial_no_version_fail.db";
    let (_db,app) = setup(DB_NAME).await?;

    let request = Request::builder()
        .uri(format!("/package/{}", WRONG_PACKAGE_NAME))
        .method("GET")
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();

    assert_eq!(status, StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn get_package_initial_file_success() -> Result<()> {
    dotenv().ok();
    const DB_NAME: &str = "db/test_initial_package.db";
    let (_db,app) = setup(DB_NAME).await?;

    let request = Request::builder()
        .uri(format!("/package/{}/{}", PACKAGE_NAME, VERSION))
        .method("GET")
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();

    let bytes = response.into_body().collect().await?.to_bytes();
    if status != StatusCode::OK {
        println!("status: {}", status);
        println!("body: {:?}", String::from_utf8_lossy(&bytes));
        panic!("Wrong status code");
    }
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;
    assert_eq!(status, StatusCode::OK);
    let repo_id = body.get("repo_id").unwrap();
    let version = body.get("version").unwrap();
    let file_hash = body.get("file_hash").unwrap();
    let file_path = body.get("file_path").unwrap();
    let code = body.get("code").unwrap();
    assert_eq!(repo_id, 1);
    assert_eq!(version, VERSION);
    assert_eq!(file_hash, FILE_HASH);
    assert_eq!(file_path, FILE_PATH);
    assert_eq!(code, CODE);

    Ok(())
}


#[tokio::test]
async fn get_package_file_success() -> Result<()> {
    dotenv().ok();
    const DB_NAME: &str = "db/test_get_package_file_success.db";
    let (_db,app) = setup(DB_NAME).await?;

    let request = Request::builder()
        .uri(format!("/package/fetch_file/{}/{}", VERSION_ID, FILE_PATH))
        .method("GET")
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();

    let bytes = response.into_body().collect().await?.to_bytes();
    if status != StatusCode::OK {
        println!("status: {}", status);
        println!("body: {:?}", String::from_utf8_lossy(&bytes));
        panic!("Wrong status code");
    }
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;
    assert_eq!(status, StatusCode::OK);
    let header_id = body.get("header_id").unwrap();
    let file_path = body.get("file_path").unwrap();
    let file_hash = body.get("file_hash").unwrap();
    let code = body.get("code").unwrap();
    assert_eq!(header_id, VERSION_ID);
    assert_eq!(file_path, FILE_PATH);
    assert_eq!(file_hash, FILE_HASH);
    assert_eq!(code, CODE);

    Ok(())
}


