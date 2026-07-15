use std::sync::Arc;

use anyhow::Ok;
use anyhow::Result;
use axum::Router;

use crate::{
    db::sqlite_db::SqliteDb,
    models::app_state::{AppState, AppStateBuilder},
};

pub mod db;
pub mod models;
pub mod repos;
pub mod routes;
pub mod service;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting server...");

    let db = Arc::new(SqliteDb::new().await?);
    db.migrate().await?;
    let state = AppStateBuilder::new().db_provider(db).build()?;
    let app = Router::new().merge(routes::router()).with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Listening on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
