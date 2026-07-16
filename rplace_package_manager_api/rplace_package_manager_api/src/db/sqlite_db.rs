use std::{env, str::FromStr};

use anyhow::{Ok, Result};
use dotenvy::dotenv;
use sqlx::{SqlitePool, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};
use std::fs;

use crate::db::db_provider::Repo;

#[derive(Debug)]
pub struct SqliteDb {
    pub pool: SqlitePool,
}
impl SqliteDb {
    pub async fn new_with_db_url<T:ToString>(database_url: T) -> Result<Self>{
        let database_url = database_url.to_string();
        let options = SqliteConnectOptions::from_str(&database_url)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }
    pub async fn new() -> Result<Self> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("current_dir: {:?}", std::env::current_dir()?);
        println!("database_url: {:?}", database_url);

        return Self::new_with_db_url(database_url).await;
    }
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
impl Repo for SqliteDb {
    
}
