use std::sync::{Arc, LazyLock, RwLock};

use clap::{Arg, CommandFactory, Parser, error::ErrorKind};

pub enum ArgOptions {
    Parse (ParseArgs),
    ReloadConfig,
}
#[derive(Debug)]
pub struct ParseArgs{
    pub origin: String,
    pub target: Option<String>,
}

#[derive(Parser, Debug)]
pub struct Args {
    pub origin: Option<String>,
    pub target: Option<String>,

    #[arg(long = "reload-config", short = 'r')]
    pub reload_config: bool,
}
pub fn handle_args() -> ArgOptions {
    let args = Args::parse();

    if args.reload_config {
        return ArgOptions::ReloadConfig;
    }

    if args.origin.is_none() {
        Args::command()
            .error(
                ErrorKind::MissingRequiredArgument,
                "the following required arguments were not provided:\n  <ORIGIN>",
            )
            .exit();
    }
    return ArgOptions::Parse (ParseArgs { origin: args.origin.unwrap(), target: args.target });
}