use std::io::Write;
use std::fs::OpenOptions;
use std::process::exit;

use crate::term::data_providers::{DataSouce, TextProvider};
use crate::writer::WriterResult;
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
    let mut replaced: WriterResult = writer.replace();

    let mut write_path = match args.target {
        Some(path) => path,
        None => args.origin,
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(&write_path)
        .expect("Unable to open file");

    let last = replaced.file_data.pop().unwrap();
    write!(&mut file, "{}", last.data).expect("Unable to write");

    replaced.file_data.iter_mut().for_each(|result| {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&result.path)
            .expect(&format!("Unable to open or create file {}", result.path));
        println!("data for file {}: {}", result.path,result.data);
        write!(&mut file, "{}", result.data).expect("Unable to write");
    });
}
