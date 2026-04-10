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

    for elem in &nodes {
        println!("{:?}", elem);
    }

    let writer = Writer::new(nodes);
    let replaced = writer.replace();
    println!("replaced: {}",replaced);
}

/*- def a:

    pub struct $#struct_name {
        str: String
    }
    pub impl $#struct_name {
        pub fn new() -> Self{
            Self {
                "Hello world from $#struct_name !".to_string(),
            }
        }
        pub fn print(&self){
            println!("{}",self.str);
        }
    }

*///- endef: