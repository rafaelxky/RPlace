use crate::{db::db_provider::DbProvider, models::package::{PackageAccessDto, PackageCreateDto, PackagePublicDto}};

use anyhow::Result;

pub fn insert_package(package: PackageCreateDto, db: &dyn DbProvider) -> Result<()>{
    todo!()
}
pub fn get_package(data: PackageAccessDto, db: &dyn DbProvider) -> Result<PackagePublicDto>{
    todo!()
}
pub fn update_package(package: PackageCreateDto, dn: &dyn DbProvider) -> Result<PackagePublicDto>{
    todo!()
}
pub fn new_package_version(package: PackageCreateDto, db: &dyn DbProvider) -> Result<()>{
    todo!()
}
pub fn get_latest_package(name: String, db: &dyn DbProvider) -> Result<PackagePublicDto>{
    todo!()
}
pub fn delete_package(name: String, db:&dyn DbProvider) -> Result<()>{
    todo!()
}
pub fn delete_package_version(package: PackageAccessDto, db: &dyn DbProvider) -> Result<()>{
    todo!()
}