use std::path::Path;

use crate::constants::PROJECT_FILE;
use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

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
    Package {
        #[command(subcommand)]
        command: PackageSubcommand,
    },
}
#[derive(Subcommand, Debug)]
pub enum PackageSubcommand {
    Login {
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
    Run(ParseArgs),
    ReloadConfig,
    New {
        project_name: String,
    },
    Push {},
    Login {
        email: String,
        password: String,
    },
    CreateUser {
        username: String,
        email: String,
        password: String,
    },
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
        Some(SubCommand::Run {
            origin,
            target,
            stops_at_parser,
        }) => {
            let path = Path::new(PROJECT_FILE);
            if origin.is_none() && !path.is_file() {
                Args::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "the following required arguments were not provided:\n  <ORIGIN>",
                    )
                    .exit();
            }
            return CliArgs::Run(ParseArgs {
                origin,
                target,
                stops_at_parser,
            });
        }
        Some(SubCommand::Reload) => return CliArgs::ReloadConfig,
        Some(SubCommand::Package { command }) => match command {
            PackageSubcommand::Login { email, password } => {
                return CliArgs::Login { email, password };
            }
            PackageSubcommand::CreateUser {
                username,
                email,
                password,
            } => {
                return CliArgs::CreateUser { username, email, password };
            }
            PackageSubcommand::Push {} => return CliArgs::Push {},
        },
        None => {
            panic!("Invalid subcommand!")
        }
    }
}
