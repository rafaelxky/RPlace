use std::{fs::{File, OpenOptions}, path::Path, process::exit};

use clap::{Arg, Parser};

use crate::{lexer::Lexer, parser::Parser as Ps, term::data_providers::TextProvider, writer::Writer};
use std::io::Write;

#[derive(Parser, Debug)]
pub struct Args {
    origin: String,
    target: String,
}
pub fn handle_args() {
    let args = Args::parse();
    parse_lang(&args);
}
fn parse_lang(args: &Args) {
    let data = TextProvider::get_text(&args.origin);
    let lexer = Lexer::new(args.origin.clone(),data);
    let tokens = lexer.parse();
    let parser = Ps::new(tokens);
    let nodes = parser.parse();
    let writer = Writer::new(nodes);
    let replaced = writer.replace();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.origin.clone())
        .expect("Unable to open or create file");
    write!(&mut file, "{}", replaced).expect("Unable to write");
}
