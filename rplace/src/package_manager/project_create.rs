use std::{fs::{File}, io::Write};
use anyhow::{Result};

use crate::{constants::PROJECT_FILE, package_manager::package_data::PackageData};

pub fn create_project(project_name: String) -> Result<()>{
    let mut file = File::create(PROJECT_FILE)?;
    let config = PackageData::new(project_name.as_str(), "1.0.0.0", "src/your_root_file");
    let toml = toml::to_string(&config)?;
    file.write_all(&toml.into_bytes())?;
    Ok(())
}