use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::{lexer::Lexer, parser::Parser, writer::Writer, term::terminal_handler::handle_args};

pub mod error_handler;
pub mod lexer;
pub mod parser;
pub mod writer;
pub mod term;

fn main() {

    println!("Started");
    handle_args();

}

