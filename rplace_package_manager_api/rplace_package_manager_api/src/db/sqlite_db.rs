use std::env;

use anyhow::{Ok, Result};
use dotenvy::dotenv;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::db::db_provider::DbProvider;

#[derive(Debug)]
pub struct SqliteDb {
    pool: SqlitePool,
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
    pub async fn migrate(&self) -> Result<()>{
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
impl DbProvider for SqliteDb {}
