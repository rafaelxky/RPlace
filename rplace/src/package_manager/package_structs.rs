use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::config::PackageManagerCompilerConfig;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageData {
    pub package: Package,
    pub dependencies: Option<HashMap<String, Dependency>>,
    pub config: Option<PackageManagerCompilerConfig>,
}
impl PackageData {
    pub fn from_toml<T: Into<String>>(toml: T) -> Result<Self> {
        let data: PackageData = toml::from_str(&toml.into())?;
        Ok(data)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub root: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed { version: String },
}
impl Dependency {
    pub fn new_simple(version: String) -> Self {
        Self::Simple(version)
    }
    pub fn get_version(&self) -> &String {
        match self {
            Dependency::Simple(version) => version,
            Dependency::Detailed { version } => version,
        }
    }
    pub fn get_version_owned(self) -> String {
        match self {
            Dependency::Simple(version) => version,
            Dependency::Detailed { version } => version,
        }
    }
}
impl Package {
    pub fn new<T: ToString>(project_name: T, name: T, root: T) -> Self {
        let project_name = project_name.to_string();
        let version = name.to_string();
        let root = root.to_string();
        Self {
            name: project_name,
            version,
            root,
        }
    }
}
impl PackageData {
    pub fn new<T: ToString>(project_name: T, version_name: T, root: T) -> Self {
        Self {
            package: Package::new(project_name, version_name, root),
            config: None,
            dependencies: None,
        }
    }
    pub fn add_dependency(&mut self, name: String, version: String) {
        let dep: &mut HashMap<String, Dependency> = match &mut self.dependencies {
            Some(d) => d,
            None => {
                self.dependencies = Some(HashMap::new());
                self.dependencies.as_mut().unwrap()
            }
        };
        dep.insert(name, Dependency::new_simple(version));
    }
}
