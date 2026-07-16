use std::{collections::HashMap, sync::Arc};

use anyhow::{Result};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use dotenvy::dotenv;
use http_body_util::BodyExt;
use rand_core::OsRng;
use rplace_package_manager_api::{
    app::app,
    db::{sqlite_db::SqliteDb},
    models::user::user::{User, UserPublicDto},
};
use tower::ServiceExt;
use std::result::Result::Ok;


#[tokio::test]
async fn test_new_user() -> Result<()> {
    dotenv().ok();
    let _ = std::fs::remove_file("test_new_user_db.db");
    let db = SqliteDb::new_with_db_url("test_new_user_db.db").await?;
    db.migrate().await?;
    let db: Arc<SqliteDb> = Arc::new(db);
    let app = app(db.clone()).await?;

    let request = Request::builder()
        .uri("/user")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{
            "name": "rafael",
            "email": "example@gmail.com",
            "password": "password123"
        }"#,
        ))?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();

    let bytes = response.into_body().collect().await?.to_bytes();
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;
        let msg = body.get("message").unwrap();
        let err = body.get("err").unwrap();
        println!("msg: {}", msg);
        println!("err: {}", err);
    }
    assert_eq!(status, StatusCode::OK);
    let user: UserPublicDto = serde_json::from_slice(&bytes)?;
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "rafael");

    let sql = "SELECT * FROM users WHERE id = 1;";
    let _user = sqlx::query_as::<_, User>(sql).fetch_one(&db.pool).await?;

    Ok(())
}

#[tokio::test]
async fn test_loggin() -> Result<()> {
    dotenv().ok();
    let _ = std::fs::remove_file("test_loggin_db.db");
    let db = SqliteDb::new_with_db_url("test_loggin_db.db").await?;
    db.migrate().await?;
    let db: Arc<SqliteDb> = Arc::new(db);
    let app = app(db.clone()).await?;

    let salt = SaltString::generate(&mut OsRng);

    let password = "password123";
    let email = "example@gmail.com";
    let password_hash = Argon2::default().hash_password(password.as_bytes(), &salt).expect("could not hash password").to_string();
    let sql = "INSERT INTO users (name,email,password_hash) VALUES (?,?,?);";
    sqlx::query(sql).bind("rafael2").bind(email).bind(password_hash).execute(&db.pool).await?;

      let request = Request::builder()
        .uri("/loggin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            "{{
            \"email\": \"{}\",
            \"password\": \"{}\"
        }}",email,password)
        ))?;

    let response = app.oneshot(request).await?;
    let status = response.status().clone();
    let bytes = response.into_body().collect().await?.to_bytes();
    let body: HashMap<String, serde_json::Value> = serde_json::from_slice(&bytes.clone())?;
    if status != StatusCode::OK {
        let msg = body.get("message").unwrap();
        let err = body.get("err").unwrap();
        println!("msg: {}", msg);
        println!("err: {}", err);
    }
    assert_eq!(status, StatusCode::OK);
    let _tok = body.get("token").expect("did not return JWT token");

    Ok(())
}