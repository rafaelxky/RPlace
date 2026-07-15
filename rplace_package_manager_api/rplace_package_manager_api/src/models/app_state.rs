use std::sync::Arc;
use anyhow::{Result, anyhow};

use crate::db::db_provider::{self, PackageRepo};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db_provider: Arc<dyn PackageRepo>
}
impl AppState {
    pub fn new(db_provider: Arc<dyn PackageRepo>) -> Self{
        Self { db_provider }
    }
}
pub struct AppStateBuilder{
    db_provider: Option<Arc<dyn PackageRepo>>
}
impl AppStateBuilder {
    pub fn new() -> Self{ 
        Self { db_provider: None }
    }
    pub fn build(self) -> Result<AppState>{
        let Some(db_provider) = self.db_provider else {
            return Err(anyhow!("DB provider was not provided for app state!"));
        };
        Ok(AppState::new(db_provider))
    }
    pub fn db_provider(mut self, db_provider: Arc<dyn PackageRepo>) -> Self{
        self.db_provider = Some(db_provider);
        self
    }
}