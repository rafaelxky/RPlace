use std::{
    fs::{self, File},
    io::Write,
};

use crate::{
    config::config::{CompilerConfig, PackageManagerCompilerConfig},
    constants::PROJECT_FILE,
    package_manager::package_structs::{Package, PackageData},
    term::terminal_handler::ParseArgs,
};
use anyhow::{Ok, Result};

pub fn get_package_manager_data() -> Result<PackageData> {
    let file = fs::read_to_string("rplace.toml")?;
    let data: PackageData = toml::from_str(&file)?;
    Ok(data)
}
pub fn join_args_and_config(
    args: ParseArgs,
    package_manager_data: &PackageData,
    config: CompilerConfig,
) -> (ParseArgs, CompilerConfig) {
    let mut config = config;
    let args = join_package(args, package_manager_data.package.clone());
    match &package_manager_data.config {
        Some(c) => {
            config = join_config(config, c);
        }
        _ => (),
    }
    (args, config)
}
fn join_package(args: ParseArgs, package: Package) -> ParseArgs {
    let mut args = args;
    args.origin = Some(package.root);
    args
}
fn join_config(config: CompilerConfig, package: &PackageManagerCompilerConfig) -> CompilerConfig {
    let mut config = config;
    match package.allow_import {
        Some(b) => config.allow_import = b,
        _ => (),
    }
    match package.allow_lua {
        Some(b) => config.allow_lua = b,
        None => (),
    }
    match package.package_source.clone() {
        Some(s) => config.package_source = s,
        None => (),
    }
    config
}

pub fn create_project(project_name: String) -> Result<()> {
    let mut file = File::create(PROJECT_FILE)?;
    let config = PackageData::new(project_name.as_str(), "1.0.0.0", "src/your_root_file");
    let toml = toml::to_string(&config)?;
    file.write_all(&toml.into_bytes())?;
    Ok(())
}
pub fn write_to_project(package_data: PackageData) -> Result<()> {
    let mut file = File::create(PROJECT_FILE)?;
    let toml = toml::to_string(&package_data)?;
    file.write_all(&toml.into_bytes())?;
    Ok(())
}
