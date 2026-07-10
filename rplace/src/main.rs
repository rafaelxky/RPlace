use std::collections::HashMap;
use std::fs::OpenOptions;
use std::process::exit;
use std::sync::{Arc, RwLock};

use crate::config::config::CONFIG;
use crate::data_stream::{DataSouce, get_data_stream};
use crate::output_stream::OutputWriter;
use crate::structs::FileConfig;
use crate::writer::writer::Writer;
use crate::writer::writer_structs::WriterResult;
use crate::{lexer::Lexer, parser::Parser, term::terminal_handler::handle_args};

pub mod config;
pub mod data_stream;
pub mod derive;
pub mod error_handler;
pub mod lexer;
pub mod lua;
pub mod options;
pub mod output_stream;
pub mod parser;
pub mod structs;
pub mod term;
pub mod writer;

fn main() {
    let args = handle_args();
    let (mut stream, origin) = get_data_stream(&args.origin);
    match origin {
        DataSouce::WEB => {
            if args.target.is_none() {
                eprintln!("No target file specified for web data souce");
                exit(1);
            }
        }
        DataSouce::FILE => (),
    }

    // todo fix target path to create subfolders
    // todo make so that derive can create folders
    // todo fix paths inside of the folder to reference according to folder path instead of program execution
    // todo avoid access to upper folders from
    // fix imports check b.txt
    // fix import space between : and ident not working
    // tests
    let imports = Arc::new(RwLock::new(HashMap::new()));
    loop {
        let data = stream.next();
        if data.is_none() {
            break;
        }
        let (data, path) = data.unwrap();
        let lexer = Lexer::new(path.clone(), data);
        let tokens = lexer.parse();
        let parser = Parser::new(tokens);
        let nodes = parser.parse();
        let writer = Writer::new_with_imports(nodes, imports.clone());
        let (replaced, config): (WriterResult, FileConfig) = writer.replace();

        let file = match (&args.target, &config.output) {
            (Some(path), _) => OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .expect("Unable to open file"),
            (None, Some(file_path_config)) => OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path_config.clone())
                .expect("Unable to open file"),
            (None, None) => OpenOptions::new()
                .write(true)
                .create(false)
                .truncate(true)
                .open(path)
                .expect("Unable to open file"),
        };

        let output = OutputWriter::new(replaced, file, config);
        output.write();
    }
}
