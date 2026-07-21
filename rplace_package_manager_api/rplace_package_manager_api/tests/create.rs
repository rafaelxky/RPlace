use anyhow::{Ok, Result};
use axum::{
    Router,
    http::{Request, StatusCode},
    routing::Route,
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use rplace_package_manager_api::{
    app::app, db::sqlite_db::SqliteDb, models::{
        link::link::Link, loggin::loggin::JwtClaims, package_file::package_file::PackageFile, package_version_header::package_version_header::PackageVersionHeader, registry::package_registry::PackageRegistry,
    },
};
use std::{env::var, sync::Arc};
use tower::ServiceExt;

const PACKAGE_NAME: &str = "my_package";
const VERSION: &str = "1.0.0.0";
const CODE: &str = "println(\"Hello World!\")";
const PATH: &str = "src/main.rs";

const JSON_HEADER_KEY: &str = "Content-Type";
const JSON_HEADER_VALUES: &str = "application/json";
const AUTH_HEADER_KEY: &str = "Authorization";

const INSERT_USER: &str = "INSERT INTO users (name,email,password_hash) VALUES (?,?,?);";
const GET_PACKAGE_HEADER: &str = "SELECT * FROM package_registry WHERE id = ?;";
const INSERT_PACKAGE_HEADER: &str =
    "INSERT INTO package_registry (package_name, created_at, creator_id) VALUES (?,?,?);";
const GET_VERSION_HEADER: &str = "SELECT * FROM package_version_header WHERE id = ?;";
const INSERT_VERSION_HEADER: &str =
    "INSERT INTO package_version_header (version, created_at, package_id) VALUES (?,?,?);";
const GET_FILE: &str = "SELECT * FROM package_file WHERE code = ?;";
const GET_LINK: &str = "SELECT * FROM links WHERE package_version_id = 1;";

// package header repeated name
// package version repeated version
// package version wrong loggin

async fn setup(db_name: &str) -> Result<(Arc<SqliteDb>, Router)> {
    dotenv().ok();
    let _ = std::fs::remove_file(db_name);
    let db = SqliteDb::new_with_db_url(db_name).await?;
    db.migrate().await?;
    let db: Arc<SqliteDb> = Arc::new(db);
    let app = app(db.clone()).await?;

    sqlx::query(INSERT_USER)
        .bind("usrA")
        .bind("example@gmail.com")
        .bind("abc")
        .execute(&db.pool)
        .await?;

    Ok((db, app))
}
async fn setup_user() -> Result<String> {
    let secret = &var("JWT_SECRET")?.into_bytes();
    let claims = JwtClaims {
        user_id: 1,
        expiration_date: 4_000_000_000,
    };

    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;
    return Ok(jwt);
}
// creates a package header in the db
async fn setup_package_header(db_name: &str) -> Result<(String, Arc<SqliteDb>, Router)> {
    let (db, app) = setup(db_name).await?;
    let secret = &var("JWT_SECRET")?.into_bytes();
    let claims = JwtClaims {
        user_id: 1,
        expiration_date: 4_000_000_000,
    };

    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;

    sqlx::query(INSERT_PACKAGE_HEADER)
        .bind(PACKAGE_NAME)
        .bind(1)
        .bind(1)
        .execute(&db.pool)
        .await?;

    return Ok((jwt, db, app));
}
// creates a package header and version header in db
async fn setup_file(db_name: &str) -> Result<(String, Arc<SqliteDb>, Router)> {
    let (tok, db, app) = setup_package_header(db_name).await?;
    sqlx::query(INSERT_VERSION_HEADER)
        .bind(VERSION)
        .bind(1)
        .bind(1)
        .execute(&db.pool)
        .await?;

    Ok((tok, db, app))
}

#[tokio::test]
async fn package_create_success() -> Result<()> {
    let (db, app) = setup("db/package_create_success.db").await?;

    let json = format!(
        "
    {{ 
        \"name\": \"{}\"
    }}
        ",
        PACKAGE_NAME
    );

    let tok = setup_user().await?;

    let request = Request::builder()
        .uri("/package")
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .header(AUTH_HEADER_KEY, format!("Bearer {}", tok))
        .method("POST")
        .body(json)?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();
    let bytes = response.into_body().collect().await?.to_bytes();
    if status != StatusCode::OK {
        println!("body: {:?}", String::from_utf8_lossy(&bytes));
        println!("token: {}", tok);
    }
    assert_eq!(status, StatusCode::OK);

    let pak = sqlx::query_as::<_, PackageRegistry>(GET_PACKAGE_HEADER)
        .bind(1)
        .fetch_optional(&db.pool)
        .await?;
    assert!(pak.is_some());
    let pak = pak.unwrap();
    assert_eq!(pak.package_name, PACKAGE_NAME);

    Ok(())
}
#[tokio::test]
async fn package_version_create_success() -> Result<()> {
    let (tok, db, app) = setup_package_header("db/package_version_create_success.db").await?;
    let json = format!(
        "
        {{
            \"package_name\": \"{}\",
            \"version\": \"{}\"
        }}
        ",
        PACKAGE_NAME, VERSION
    );

    let request = Request::builder()
        .uri("/package/version")
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .header(AUTH_HEADER_KEY, format!("Bearer {tok}"))
        .method("POST")
        .body(json)?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();
    let bytes = response.into_body().collect().await?.to_bytes();

    if status != StatusCode::OK {
        println!("body: {:?}", String::from_utf8_lossy(&bytes));
        println!("token: {}", tok);
    }
    assert_eq!(status, StatusCode::OK);
    let version_header = sqlx::query_as::<_, PackageVersionHeader>(GET_VERSION_HEADER)
        .bind(1)
        .fetch_optional(&db.pool)
        .await?;

    assert!(version_header.is_some());
    let version_header = version_header.unwrap();
    assert_eq!(version_header.id, 1);
    assert_eq!(version_header.version, VERSION);
    assert_eq!(version_header.package_id, 1);

    Ok(())
}

#[tokio::test]
async fn file_create_success() -> Result<()> {
    let (tok, db, app) = setup_file("db/file_created_success.db").await?;

    let json = serde_json::json!({
        "registry_id": 1,
        "version_header_id": 1,
        "code": CODE,
        "path": PATH
    })
    .to_string();

    let request = Request::builder()
        .uri("/file")
        .header(JSON_HEADER_KEY, JSON_HEADER_VALUES)
        .header(AUTH_HEADER_KEY, format!("Bearer {tok}"))
        .method("POST")
        .body(json)?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();
    let bytes = response.into_body().collect().await?.to_bytes();

    if status != StatusCode::OK {
        println!("body: {:?}", String::from_utf8_lossy(&bytes));
        println!("token: {}", tok);
    }
    assert_eq!(status, StatusCode::OK);

    let file = sqlx::query_as::<_, PackageFile>(GET_FILE)
        .bind(CODE)
        .fetch_optional(&db.pool)
        .await?;
    let link = sqlx::query_as::<_, Link>(GET_LINK)
        .fetch_optional(&db.pool)
        .await?;

    assert!(file.is_some());
    assert!(link.is_some());

    let file = file.unwrap();
    let link = link.unwrap();

    assert_eq!(file.code, CODE);
    assert_eq!(link.file_path, PATH);
    assert_eq!(link.package_version_id, 1);

    Ok(())
}
