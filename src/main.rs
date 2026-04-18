use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::exit;

use crate::term::data_providers::{DataSouce, TextProvider};
use crate::{lexer::Lexer, parser::Parser, term::terminal_handler::handle_args, writer::Writer};

pub mod error_handler;
pub mod lexer;
pub mod parser;
pub mod term;
pub mod writer;

fn main() {
    let args = handle_args();
    let (data, origin) = TextProvider::get_text(&args.origin);
    match origin {
        DataSouce::WEB => {
            if args.target.is_none() {
                eprintln!("No target file specified for web data souce");
                exit(1);
            }
        }
        DataSouce::FILE => (),
    }
    let lexer = Lexer::new(args.origin.clone(), data);
    let tokens = lexer.parse();
    let parser = Parser::new(tokens);
    let nodes = parser.parse();
    let writer = Writer::new(nodes);
    let replaced = writer.replace();

    let write_path = match &args.target {
        Some(path) => path,
        None => &args.origin,
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(write_path)
        .expect("Unable to open or create file");
    write!(&mut file, "{}", replaced).expect("Unable to write");
}
