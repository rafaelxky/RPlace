use std::fs;

use crate::{config::{config::{CompilerConfig, PackageManagerCompilerConfig}}, package_manager::package_data::{Package, PackageData}, term::terminal_handler::ParseArgs};
use anyhow::{Ok, Result};

pub fn get_package_manager_data() -> Result<PackageData>{
    let file = fs::read_to_string("rplace.toml")?;
    let data: PackageData = toml::from_str(&file)?;
    Ok(data)
}
pub fn join_args_and_config(args: ParseArgs, package_manager_data: PackageData, config: CompilerConfig) -> (ParseArgs,CompilerConfig){
    let mut config = config;
    let args = join_package(args, package_manager_data.package);
    match package_manager_data.config {
        Some(c) => {
            config = join_config(config, c);
        },
        _ => ()
    }
    (args,config)
}
fn join_package(args:ParseArgs, package: Package) -> ParseArgs{
    let mut args = args;
    args.origin = Some(package.root);
    args
}
fn join_config(config: CompilerConfig, package: PackageManagerCompilerConfig) -> CompilerConfig{
    let mut config = config;
    match package.allow_import {
        Some(b) => config.allow_import = b,
        _ => (),
    }
    match package.allow_lua {
        Some(b) => config.allow_lua = b,
        None => (),
    }
    match package.package_source {
        Some(s) => config.package_source = s,
        None => (),
    }
    config
}