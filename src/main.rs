use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::exit;

use crate::deriver::Deriver;
use crate::term::data_providers::{DataSouce, TextProvider};
use crate::writer::WriterResult;
use crate::{lexer::Lexer, parser::Parser, term::terminal_handler::handle_args, writer::Writer};

pub mod deriver;
pub mod error_handler;
pub mod lexer;
pub mod parser;
pub mod term;
pub mod writer;
pub mod derive_options;

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

    let mut file = match args.target {
        Some(path) => OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .expect("Unable to open file"),
        None => OpenOptions::new()
            .write(true)
            .create(false)
            .truncate(true)
            .open(args.origin)
            .expect("Unable to open file"),
    };

    let last: writer::FileData = replaced.file_data.pop().unwrap();
    write!(&mut file, "{}", last.data).expect("Unable to write");

    replaced.file_data.iter_mut().for_each(|result| {
        let path = Path::new(&result.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Unable to create directories");
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&result.path)
            .expect(&format!("Unable to open or create file {}", result.path));

        file.write_all(result.data.as_bytes())
            .expect("Unable to write");

    });

    replaced.derives.iter().for_each(|derive| {
        if fs::exists(&derive.path).is_err() {
            panic!("Error: no such file {} for derive", derive.path)
        }
        let result = Deriver::derive(derive);
        let mut file = OpenOptions::new()
            .write(true)
            .create(false)
            .truncate(true)
            .open(&derive.path)
            .expect("Unable to open file");
        file.write_all(result.as_bytes())
            .expect("Unable to write to file!");
    });
}
