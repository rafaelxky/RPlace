use async_trait::async_trait;
use anyhow::{Ok, Result};

use crate::{db::{db_provider::PackageRegistryRepo, sqlite_db::SqliteDb}, models::package_registry::{PackageRegistry, PackageRegistryPublicDto}};

#[async_trait]
impl PackageRegistryRepo for SqliteDb {
    
}
