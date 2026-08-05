use crate::config::config::{CONFIG, CompilerConfig, reload_config};
use crate::package_manager::auth::save_tok;
use crate::package_manager::package_data::PackageData;
use crate::package_manager::package_load::{get_package_manager_data, join_args_and_config};
use crate::package_manager::package_upload::upload_files;
use crate::package_manager::project_create::create_project;
use crate::package_manager::web::auth::loggin;
use crate::package_manager::web::fetch::get_initial_package_version;
use crate::package_manager::web::user::create_user;
use crate::run::run_options::run_parse;
use crate::term::terminal_handler::handle_args;
use crate::term::terminal_handler::{CliArgs, ParseArgs};
use anyhow::Result;
use directories::ProjectDirs;
use crate::package_manager::auth::read_tok;
use crate::package_manager::web::create::{create_new_package, create_new_version};
use crate::run::run_options::parse_get_all_paths;
use std::process::exit;

pub mod config;
pub mod constants;
pub mod data_stream;
pub mod derive;
pub mod error_handler;
pub mod lexer;
pub mod lua;
pub mod options;
pub mod output_stream;
pub mod package_manager;
pub mod parser;
pub mod run;
pub mod structs;
pub mod term;
pub mod writer;
pub mod errors;

#[tokio::main]
async fn main() -> Result<()> {
    let args = handle_args();
    match args {
        CliArgs::New { project_name } => {
            create_project(project_name)?;
            Ok(())
        }
        CliArgs::Run(args) => {
            let data = get_package_manager_data();
            let config = CONFIG.clone().read().unwrap().clone();
            let (args, config): (ParseArgs, CompilerConfig) = match data {
                Ok(d) => join_args_and_config(args, d, config),
                Err(_e) => (args, config),
            };
            let stops_at_parser = args.stops_at_parser;
            let to_write = run_parse(args, config);
            if stops_at_parser {
                return Ok(());
            }
            to_write.into_iter().for_each(|output| {
                output.write();
            });
            Ok(())
        }
        CliArgs::ReloadConfig => {
            let dir = ProjectDirs::from("io", "rplace", "rplace");
            let dir = match dir {
                Some(dir) => dir,
                None => {
                    println!("Unable to find config path");
                    exit(0);
                }
            };
            let config = dir.config_dir().join("config.json");
            reload_config(config);
            println!("Config reloaded successfully!");
            Ok(())
        }
        CliArgs::Login { email, password } => {
            let config: CompilerConfig = CONFIG.clone().read().unwrap().clone();
            let res = loggin(&config.package_source, &email, &password).await?;
            save_tok(res)?;
            println!("Successull loggin!");
            Ok(())
        }
        CliArgs::Push {} => {
            let data: PackageData = get_package_manager_data()?;
            let config = CONFIG.clone().read().unwrap().clone();
            let package_source = config.package_source.clone();
            let package_name = data.package.name.clone();
            let package_version = data.package.version.clone();
            let header = get_initial_package_version(
                &package_source,
                &package_name,
                &package_version,
            )
            .await?;
            let token = read_tok()?;
            let paths = parse_get_all_paths(data, config);

            upload_files(paths, &package_source,header.repo_id, header.header_id, &token.token).await?;

            Ok(())
        }
        CliArgs::CreateUser {
            username,
            email,
            password,
        } => {
            let config = CONFIG.clone().read().unwrap().clone();
            let user = create_user(&config.package_source, &username, &email, &password).await?;
            println!("User {} create successfully!", user.name);
            Ok(())
        }
        CliArgs::NewPackage {} => {
            let config = CONFIG.clone().read().unwrap().clone();
            let data = get_package_manager_data()?;
            let package_name = data.package.name;
            let package_version = data.package.version;
            let token = read_tok()?;
            let package =
                create_new_package(&config.package_source, &package_name, &token.token).await?;
            let version = create_new_version(
                &config.package_source,
                &package_name,
                &package_version,
                &token.token,
            )
            .await?;
            println!(
                "Created new package {} and version {}",
                package.name, version.version
            );
            Ok(())
        }
        CliArgs::NewVersion {} => {
            let config = CONFIG.clone().read().unwrap().clone();
            let data = get_package_manager_data()?;
            let token = read_tok()?;
            let package_name = data.package.name;
            let package_version = data.package.version;
            let res = create_new_version(
                &config.package_source,
                &package_name,
                &package_version,
                &token.token,
            )
            .await?;
            println!(
                "Created new package version {} for package {}",
                res.version, package_name
            );
            Ok(())
        }
    }
}
