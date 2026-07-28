use std::path::Path;

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

use crate::constants::PROJECT_FILE;

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    New {
        project_name: String,
    },
    Run {
        origin: Option<String>,
        target: Option<String>,
    },
    Reload,
    Loggin {
        email: String,
        password: String,
    },
    CreateUser {
        username: String,
        email: String,
        password: String,
    }
}
#[derive(Debug)]
pub enum CliArgs {
    Parse(ParseArgs),
    ReloadConfig,
    New { project_name: String },
    Loggin{
        email: String,
        password: String,
    }
}
#[derive(Debug)]
pub struct ParseArgs {
    pub origin: Option<String>,
    pub target: Option<String>,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}
pub fn handle_args() -> CliArgs {
    let args = Args::parse();

    match args.command {
        Some(SubCommand::New { project_name }) => {
            return CliArgs::New { project_name };
        }
        Some(SubCommand::Run { origin, target }) => {
            let path = Path::new(PROJECT_FILE);
            if origin.is_none() && !path.is_file() {
                Args::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "the following required arguments were not provided:\n  <ORIGIN>",
                    )
                    .exit();
            }
            return CliArgs::Parse(ParseArgs { origin, target });
        }
        Some(SubCommand::Reload) => {
            return CliArgs::ReloadConfig
        }
        Some(SubCommand::Loggin { email, password }) => {
            return CliArgs::Loggin { email, password };
        },
        Some(SubCommand::CreateUser { username, email, password }) => {
            todo!()   
        }
        None => {
            panic!("Invalid subcommand!")
        },
    }
}
