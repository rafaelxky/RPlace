use std::path::Path;

use crate::constants::PROJECT_FILE;
use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Create a new RPlace project.
    New {
        /// Name to use for the new project.
        #[arg(value_name = "PROJECT_NAME", help = "Name to use for the new project.")]
        project_name: String,
    },
    /// Parse and render an RPlace source file.
    Run {
        /// Source file or project to process; defaults to rplace.toml when available.
        #[arg(
            value_name = "ORIGIN",
            help = "Source file or project to process; defaults to rplace.toml when available."
        )]
        origin: Option<String>,
        /// Output file; defaults to the configured output or the source file.
        #[arg(
            value_name = "TARGET",
            help = "Output file; defaults to the configured output or the source file."
        )]
        target: Option<String>,
        /// Render an inline template instead of reading a source file.
        #[arg(
            short = 'e',
            long = "eval",
            value_name = "TEMPLATE",
            conflicts_with = "origin"
        )]
        expression: Option<String>,
        /// Parse and print the syntax tree without rendering output.
        #[arg(
            short = 'p',
            long = "parse-only",
            help = "Parse and print the syntax tree without rendering output."
        )]
        stops_at_parser: bool,
        /// Set a global template value; may be passed more than once.
        #[arg(short = 'D', long = "var", value_name = "KEY=VALUE", value_parser = parse_global_value, action = clap::ArgAction::Append)]
        global_vars: Vec<(String, String)>,
    },
    /// Reload the global RPlace configuration.
    Reload,
    /// Manage packages and package dependencies.
    Package {
        #[command(subcommand)]
        command: PackageSubcommand,
    },
    /// Render piped text using a source file as its template.
    Pipe {
        /// Template source file.
        #[arg(value_name = "TARGET", help = "Template source file.")]
        target: String,
        /// Text to pass into the template.
        #[arg(value_name = "TEXT", help = "Text to pass into the template.")]
        command: String,
    },
}
#[derive(Subcommand, Debug)]
pub enum PackageSubcommand {
    /// Log in to the configured package service.
    Login {
        /// Account email address.
        #[arg(value_name = "EMAIL", help = "Account email address.")]
        email: String,
        /// Account password.
        #[arg(value_name = "PASSWORD", help = "Account password.")]
        password: String,
    },
    /// Create a user on the configured package service.
    CreateUser {
        /// Username for the new account.
        #[arg(value_name = "USERNAME", help = "Username for the new account.")]
        username: String,
        /// Email address for the new account.
        #[arg(value_name = "EMAIL", help = "Email address for the new account.")]
        email: String,
        /// Password for the new account.
        #[arg(value_name = "PASSWORD", help = "Password for the new account.")]
        password: String,
    },
    /// Upload the current project to the package service.
    Push {},
    /// Create a package using the current project metadata.
    New {},
    /// Publish a new version of the current package.
    CreateVersion {},
    /// Add a package dependency to the current project.
    Add {
        /// Name of the package to add.
        #[arg(value_name = "PACKAGE", help = "Name of the package to add.")]
        dependency_name: String,
        /// Optional version constraint for the dependency.
        #[arg(
            value_name = "VERSION",
            help = "Optional version constraint for the dependency."
        )]
        dependency_version: Option<String>,
    },
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
    NewPackage {},
    NewVersion {},
    AddDependency {
        dependency_name: String,
        dependency_version: Option<String>,
    },
    Pipe {
        target: String,
        command: String,
    },
}
#[derive(Debug)]
pub struct ParseArgs {
    pub origin: Option<String>,
    pub target: Option<String>,
    pub expression: Option<String>,
    pub stops_at_parser: bool,
    pub global_vars: Vec<(String, String)>,
}

fn parse_global_value(value: &str) -> Result<(String, String), String> {
    let (key, value) = value
        .split_once('=')
        .ok_or_else(|| "expected KEY=VALUE".to_string())?;
    if key.is_empty() {
        return Err("global variable key cannot be empty".to_string());
    }
    Ok((key.to_string(), value.to_string()))
}

#[derive(Parser, Debug)]
#[command(
    name = "rplace",
    about = "Parse and render RPlace templates and manage RPlace projects.",
    version
)]
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
            expression,
            stops_at_parser,
            global_vars,
        }) => {
            let path = Path::new(PROJECT_FILE);
            if origin.is_none() && expression.is_none() && !path.is_file() {
                Args::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "provide <ORIGIN>, --eval <TEMPLATE>, or run inside a project",
                    )
                    .exit();
            }
            if expression.is_some() && stops_at_parser {
                Args::command()
                    .error(
                        ErrorKind::ArgumentConflict,
                        "--eval cannot be combined with --parse-only",
                    )
                    .exit();
            }
            return CliArgs::Run(ParseArgs {
                origin,
                target,
                expression,
                stops_at_parser,
                global_vars,
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
                return CliArgs::CreateUser {
                    username,
                    email,
                    password,
                };
            }
            PackageSubcommand::Push {} => return CliArgs::Push {},
            PackageSubcommand::New {} => return CliArgs::NewPackage {},
            PackageSubcommand::CreateVersion {} => return CliArgs::NewVersion {},
            PackageSubcommand::Add {
                dependency_name,
                dependency_version,
            } => {
                return CliArgs::AddDependency {
                    dependency_name,
                    dependency_version,
                };
            }
        },
        Some(SubCommand::Pipe { target, command }) => {
            return CliArgs::Pipe { target, command };
        }
        None => {
            panic!("Invalid subcommand!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, SubCommand};
    use clap::Parser;

    #[test]
    fn parses_repeated_global_values() {
        let args = Args::try_parse_from([
            "rplace",
            "run",
            "--eval",
            "Hello, $g#name!",
            "-D",
            "name=RPlace",
            "--var",
            "message=hello=world",
        ])
        .unwrap();

        let Some(SubCommand::Run {
            global_vars,
            expression,
            ..
        }) = args.command
        else {
            panic!("expected run command");
        };
        assert_eq!(expression.as_deref(), Some("Hello, $g#name!"));
        assert_eq!(
            global_vars,
            vec![
                ("name".to_string(), "RPlace".to_string()),
                ("message".to_string(), "hello=world".to_string()),
            ]
        );
    }

    #[test]
    fn rejects_global_value_without_key_value_separator() {
        let result = Args::try_parse_from(["rplace", "run", "--eval", "hello", "--var", "name"]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_source_file_and_inline_template_together() {
        let result = Args::try_parse_from(["rplace", "run", "template.rp", "--eval", "hello"]);
        assert!(result.is_err());
    }
}
