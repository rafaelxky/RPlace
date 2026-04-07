use crate::{lexer::Lexer, parser::Parser, writer::Writer};

pub mod lexer;
pub mod parser;
pub mod writer;

fn main() {
    let lexer = Lexer::new("example.txt");
    let tokens = lexer.parse();
    for elem in &tokens {
        println!("{:?}", elem);
    }
    let parser = Parser::new(tokens);
    let nodes = parser.parse();
    let writer = Writer::new(nodes);
    let replaced = writer.replace(&[("b","world!"), ("struct_name", "vec")]);
    println!("replaced: {}",replaced);
}
