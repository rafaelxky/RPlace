use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::exit;

use crate::data_stream::{DataSouce, get_data_stream};
use crate::deriver::Deriver;
use crate::writer::WriterResult;
use crate::{lexer::Lexer, parser::Parser, term::terminal_handler::handle_args, writer::Writer};

pub mod derive_options;
pub mod deriver;
pub mod error_handler;
pub mod lexer;
pub mod parser;
pub mod term;
pub mod writer;
pub mod data_stream;

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
    loop {
        let data = stream.next();
        if data.is_none() {
            break;
        } 
        let (data,path) = data.unwrap();
        let lexer = Lexer::new(path.clone(), data);
        let tokens = lexer.parse();
        let parser = Parser::new(tokens);
        let nodes = parser.parse();
        let writer = Writer::new(nodes);
        let mut replaced: WriterResult = writer.replace();

        let mut file = match &args.target {
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
                .open(path)
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
}