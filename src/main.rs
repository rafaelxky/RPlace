use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::{lexer::Lexer, parser::Parser, writer::Writer, terminal_handler::handle_args};

pub mod error_handler;
pub mod lexer;
pub mod parser;
pub mod writer;
pub mod terminal_handler;

fn main() {

    handle_args();

    /* 
    let lexer = Lexer::new("mvctest.txt");
    let tokens = lexer.parse();
    for elem in &tokens.tokens {
        println!("{:?}", elem);
    }
    let parser = Parser::new(tokens);
    let nodes = parser.parse();

    for elem in &nodes.nodes {
        println!("{:?}", elem);
    }

    let writer = Writer::new(nodes);
    let replaced = writer.replace();
    println!("replaced: {}", replaced);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("output.txt")
        .expect("Unable to open or create file");
    write!(file,"{}",replaced).expect("Unable to write");
    */
}

