use std::path::Path;

use clap::{CommandFactory, Parser, error::ErrorKind};

use crate::constants::PROJECT_FILE;

pub enum ArgOptions {
    Parse (ParseArgs),
    ReloadConfig,
    New{
        project_name: String
    },
}
#[derive(Debug)]
pub struct ParseArgs{
    pub origin: Option<String>,
    pub target: Option<String>,
}

#[derive(Parser, Debug)]
pub struct Args {
    pub origin: Option<String>,
    pub target: Option<String>,

    #[arg(long = "reload-config", short = 'r')]
    pub reload_config: bool,

    #[arg(long = "new", short = 'n', value_name = "PROJECT")]
    pub new_project: Option<String>,
}
pub fn handle_args() -> ArgOptions {
    let args = Args::parse();

    if args.reload_config {
        return ArgOptions::ReloadConfig;
    } else if args.new_project.is_some(){
        return ArgOptions::New {
            project_name: args.new_project.unwrap()
        }
    }

    let path = Path::new(PROJECT_FILE);
    if args.origin.is_none() && !path.is_file() {
        Args::command()
            .error(
                ErrorKind::MissingRequiredArgument,
                "the following required arguments were not provided:\n  <ORIGIN>",
            )
            .exit();
    }
    return ArgOptions::Parse (ParseArgs { origin: args.origin, target: args.target });
}