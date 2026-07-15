use std::fmt::Debug;
use async_trait::async_trait;
use anyhow::Result;

use crate::models::{package::{PackageAccessDto, PackageCreateDto, PackagePublicDto}, package_registry::{PackageRegistry, PackageRegistryPublicDto}};

#[async_trait]
pub trait PackageRepo: Debug + Send + Sync{
    async fn insert_package(&self, package: PackageCreateDto) -> Result<PackagePublicDto>;
    async fn get_package(&self, data: PackageAccessDto) -> Result<PackagePublicDto>;
    async fn update_package(&self, package: PackageCreateDto) -> Result<PackagePublicDto>;
    async fn get_latest_package(&self, id: i32) -> Result<PackagePublicDto>;
    async fn delete_package(&self, package_id: i32) -> Result<()>;
    async fn delete_package_version(&self, package: PackageAccessDto) -> Result<()>;
}
#[async_trait]
pub trait PackageRegistryRepo: Debug + Send + Sync{
    async fn register_package(&self, name: String, creator_id: i32) -> Result<PackageRegistry>;
    async fn get_package_registry_by_name(&self, name: String) -> Result<PackageRegistryPublicDto>;
}