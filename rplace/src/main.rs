use std::process::exit;
use directories::ProjectDirs;
use crate::config::config::{reload_config};
use crate::run::run_options::run_parse;
use crate::{term::terminal_handler::handle_args};

pub mod config;
pub mod data_stream;
pub mod derive;
pub mod error_handler;
pub mod lexer;
pub mod lua;
pub mod options;
pub mod output_stream;
pub mod parser;
pub mod run;
pub mod structs;
pub mod term;
pub mod writer;
pub mod package_manager;

fn main() {
    let args = handle_args();
    match args {
        term::terminal_handler::ArgOptions::Parse(args) => {
            run_parse(args);
        }
        term::terminal_handler::ArgOptions::ReloadConfig => {
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
        }
    }
}
