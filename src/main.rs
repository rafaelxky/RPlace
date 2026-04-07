use crate::{lexer::Lexer, parser::Parser};

pub mod lexer;
pub mod parser;

fn main() {
    let lexer = Lexer::new("example.txt");
    let tokens = lexer.parse();
    for elem in &tokens {
        println!("{:?}", elem);
    }
    let parser = Parser::new(tokens);
    let nodes = parser.parse();
}
