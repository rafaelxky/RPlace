use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{config::config::PackageManagerCompilerConfig};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PackageData{
    pub package: Package,
    pub dependencies: Option<Dependencies>,
    pub config: Option<PackageManagerCompilerConfig>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Package{
    pub name: String,
    pub version: String,
    pub root: String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Dependencies{
    dependencies: HashMap<String, Dependency>
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Dependency{
    version: String,
}
impl Package {
    pub fn new<T:ToString>(project_name: T, name: T, root: T) -> Self{
        let project_name = project_name.to_string();
        let version = name.to_string();
        let root = root.to_string();
        Self { name: project_name, version, root }
    }
}
impl PackageData {
    pub fn new<T:ToString>(project_name: T, version_name: T, root: T) -> Self{
        Self {
            package: Package::new(project_name, version_name, root),
            config: None,
            dependencies: None,
        }
    }
}