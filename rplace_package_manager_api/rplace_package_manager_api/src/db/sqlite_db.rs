use std::env;

use anyhow::{Ok, Result};
use dotenvy::dotenv;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::fs;

use crate::db::db_provider::Repo;

#[derive(Debug)]
pub struct SqliteDb {
    pub pool: SqlitePool,
}
impl SqliteDb {
    pub async fn new() -> Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        #[cfg(debug_assertions)]
        {
            if let Some(path) = database_url.strip_prefix("sqlite:") {
                use std::path::Path;
                if Path::new(path).exists() {
                    fs::remove_file(path)?;
                }
            }
        }

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
impl Repo for SqliteDb {
    
}
