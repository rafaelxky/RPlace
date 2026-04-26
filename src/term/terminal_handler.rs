use clap::{Parser};

#[derive(Parser, Debug)]
pub struct Args {
    pub origin: String,
    pub target: Option<String>,
}
pub fn handle_args() -> Args{
    let args = Args::parse();
    return args;
}
