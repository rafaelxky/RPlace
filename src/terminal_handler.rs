use std::{fs::{File, OpenOptions}, path::Path, process::exit};

use clap::{Arg, Parser};

use crate::{lexer::Lexer, parser::Parser as Ps, writer::Writer};
use std::io::Write;

#[derive(Parser, Debug)]
pub struct Args {
    path: String,
}
pub fn handle_args() {
    let args = Args::parse();
    parse_lang(&args);
}
fn parse_lang(args: &Args) {
    let path = Path::new(&args.path);
    if !Path::exists(path) {
        println!("No such file {}", args.path);
        exit(1);
    }
    let lexer = Lexer::new(args.path.clone());
    let tokens = lexer.parse();
    let parser = Ps::new(tokens);
    let nodes = parser.parse();
    let writer = Writer::new(nodes);
    let replaced = writer.replace();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.path.clone())
        .expect("Unable to open or create file");
    write!(&mut file, "{}", replaced).expect("Unable to write");
}
