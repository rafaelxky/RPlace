use std::env;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use dotenvy::dotenv;
use sqlx::{Sqlite, SqlitePool, sqlite::SqlitePoolOptions};

use crate::{db::db_provider::PackageRepo, models::package::{PackageAccessDto, PackageCreateDto, PackagePublicDto}};

#[derive(Debug)]
pub struct SqliteDb {
    pub pool: SqlitePool,
}
impl SqliteDb {
    pub async fn new() -> Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        Ok(Self { pool })
    }
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}


