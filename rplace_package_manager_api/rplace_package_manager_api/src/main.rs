use std::sync::Arc;

use anyhow::Ok;
use anyhow::Result;
use axum::Router;

use crate::app::app;
use crate::db::db_provider::Repo;
use crate::{
    db::sqlite_db::SqliteDb,
    models::app_state::{AppState, AppStateBuilder},
};

pub mod db;
pub mod models;
pub mod repos;
pub mod routes;
pub mod service;
pub mod app;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting server...");

    let db= SqliteDb::new().await?;
    db.migrate().await?;
    let db: Arc<dyn Repo> = Arc::new(db);
    
    let app = app(db).await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Listening on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
