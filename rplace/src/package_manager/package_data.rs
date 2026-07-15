use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PackageData{
    pub package: Package,
    pub dependencies: Option<Dependencies>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Package{
    name: String,
    version: String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Dependencies{
    dependencies: HashMap<String, Dependency>
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Dependency{
    version: String,
}
