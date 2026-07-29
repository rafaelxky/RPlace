use std::path::Path;

use clap::{Arg, CommandFactory, Parser, Subcommand, error::ErrorKind};

use crate::constants::PROJECT_FILE;

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    New {
        project_name: String,
    },
    Run {
        origin: Option<String>,
        target: Option<String>,
        #[arg(short = 'p')]
        stops_at_parser: bool,
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
    },
    Push {},
}
#[derive(Debug)]
pub enum CliArgs {
    Parse(
        ParseArgs,
    ),
    ReloadConfig,
    New { project_name: String },
    Loggin { email: String, password: String },
    Push {},
}
#[derive(Debug)]
pub struct ParseArgs {
    pub origin: Option<String>,
    pub target: Option<String>,
    pub stops_at_parser: bool,
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
        Some(SubCommand::Run { origin, target, stops_at_parser }) => {
            let path = Path::new(PROJECT_FILE);
            if origin.is_none() && !path.is_file() {
                Args::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "the following required arguments were not provided:\n  <ORIGIN>",
                    )
                    .exit();
            }
            return CliArgs::Parse(ParseArgs { origin, target,stops_at_parser });
        }
        Some(SubCommand::Reload) => return CliArgs::ReloadConfig,
        Some(SubCommand::Loggin { email, password }) => {
            return CliArgs::Loggin { email, password };
        }
        Some(SubCommand::CreateUser {
            username,
            email,
            password,
        }) => {
            todo!()
        }
        Some(SubCommand::Push {}) => return CliArgs::Push {},
        None => {
            panic!("Invalid subcommand!")
        }
    }
}
