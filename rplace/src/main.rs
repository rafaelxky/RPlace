use crate::config::config::{CONFIG, CompilerConfig, reload_config};
use crate::package_manager::auth::save_tok;
use crate::package_manager::package_load::{get_package_manager_data, join_args_and_config};
use crate::package_manager::project_create::create_project;
use crate::package_manager::web::auth::loggin;
use crate::run::run_options::run_parse;
use crate::term::terminal_handler::handle_args;
use crate::term::terminal_handler::{CliArgs, ParseArgs};
use anyhow::Result;
use directories::ProjectDirs;
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = handle_args();
    match args {
        CliArgs::New { project_name } => {
            create_project(project_name)?;
            Ok(())
        }
        CliArgs::Parse(args) => {
            let data = get_package_manager_data();
            let config = CONFIG.clone().read().unwrap().clone();
            let (args, config): (ParseArgs, CompilerConfig) = match data {
                Ok(d) => join_args_and_config(args, d, config),
                Err(_e) => (args, config),
            };
            run_parse(args, config);
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
        CliArgs::Loggin { email, password } => {
            let res = loggin(&email, &password).await?;
            save_tok(res)?;
            println!("User created");
            Ok(())
        }
    }
}
