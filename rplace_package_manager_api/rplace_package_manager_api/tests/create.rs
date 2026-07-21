use anyhow::{Ok, Result};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use rplace_package_manager_api::{
    app::app, db::sqlite_db::SqliteDb, models::loggin::loggin::JwtClaims,
};
use sqlx::Sqlite;
use tower::ServiceExt;
use std::{collections::HashMap, env::var, sync::Arc};

const PACKAGE_NAME: &str = "my_package";
const HEADER_KEY: &str = "Content-Type";
const HEADER_VALUES: &str = "application/json";
const INSERT_USER: &str = "INSERT INTO users (name,email,password_hash) VALUES (?,?,?);";
const AUTH_HEADER: &str = "Authorization";

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

#[tokio::test]
async fn package_create_success() -> Result<()> {
    let (_db, app) = setup("db/package_create_success.db").await?;

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
        .header(HEADER_KEY, HEADER_VALUES)
        .header(AUTH_HEADER, format!("Bearer {}", tok))
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

    Ok(())
}
