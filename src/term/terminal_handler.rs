use std::{fs::{OpenOptions}, process::exit};

use clap::{Parser};

use crate::{lexer::Lexer, parser::Parser as Ps, term::data_providers::TextProvider, writer::Writer};
use std::io::Write;

#[derive(Parser, Debug)]
pub struct Args {
    pub origin: String,
    pub target: Option<String>,
}
pub fn handle_args() -> Args{
    let args = Args::parse();
    return args;
}
