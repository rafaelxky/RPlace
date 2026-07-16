use std::fmt::Debug;
use async_trait::async_trait;
use anyhow::Result;

use crate::models::{link::link::{Link, LinkCreateDto}, package_file::package_file::{PackageFile}, package_version_header::package_version_header::PackageVersionHeader, registry::package_registry::{PackageRegistry, PackageRegistryCreateDto}, user::user::{HashedUser, User}};

#[async_trait]
pub trait Repo: 
PackageVersionHeaderRepo + 
PackageRegistryRepo + 
LinkRepo + 
PackageFileRepo + 
UserRepo +
Send + 
Sync + 
Debug {
    
}   
#[async_trait]
pub trait PackageVersionHeaderRepo: Debug + Send + Sync{
    async fn get_package_version_header_by_package_id_and_version(&self, package_id: i32, version: String) -> Result<PackageVersionHeader>;
    async fn get_latest_package_version_header_by_package_id(&self, package_id: i32) -> Result<PackageVersionHeader>;
    async fn new_package_version(&self, version: String, package_id: i32) -> Result<PackageVersionHeader>;
    async fn get_package_version_header_by_id(&self, id: i32) -> Result<PackageVersionHeader>;
}
#[async_trait]
pub trait PackageRegistryRepo: Debug + Send + Sync{
    async fn get_registry_by_name(&self, name: String) -> Result<PackageRegistry>;
    async fn get_registry_by_id(&self, id: i32) -> Result<PackageRegistry>;
    async fn new_registry(&self, registry: PackageRegistryCreateDto, user_id: i32) -> Result<PackageRegistry>;
}
#[async_trait]
pub trait LinkRepo: Debug + Send + Sync{
    async fn get_link_by_package_version_id_and_file_path(&self, package_version_id: i32, file_path: String) -> Result<Link>;
    async fn new_link(&self, link: LinkCreateDto) -> Result<Link>; 
}
#[async_trait]
pub trait PackageFileRepo: Debug + Send + Sync{
    async fn get_package_file_by_hash(&self, file_hash: String) -> Result<Option<PackageFile>>;
    async fn new_file(&self, file: PackageFile) -> Result<PackageFile>;
}
#[async_trait]
pub trait UserRepo: Debug + Send + Sync{
    async fn new_user(&self, user: HashedUser) -> Result<User>;
    async fn get_user_by_email(&self, email: String) -> Result<User>;
    async fn get_user_by_id(&self, id: i32) -> Result<User>;
}