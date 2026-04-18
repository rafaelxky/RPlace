use std::{fs::{OpenOptions}, process::exit};

use clap::{Parser};

use crate::{lexer::Lexer, parser::Parser as Ps, term::data_providers::TextProvider, writer::Writer};
use std::io::Write;

#[derive(Parser, Debug)]
pub struct Args {
    origin: String,
    target: Option<String>,
}
pub fn handle_args() {
    let args = Args::parse();
    parse_lang(&args);
}
fn parse_lang(args: &Args) {
    let (data, origin) = TextProvider::get_text(&args.origin);
    match origin {
        super::data_providers::DataSouce::WEB => {
            if args.target.is_none() {
                eprintln!("No target file specified for web data souce");
                exit(1);
            }
        },
        super::data_providers::DataSouce::FILE => {
        },
    }
    let lexer = Lexer::new(args.origin.clone(),data);
    let tokens = lexer.parse();
    let parser = Ps::new(tokens);
    let nodes = parser.parse();
    let writer = Writer::new(nodes);
    let replaced = writer.replace();

    let write_path = match &args.target {
        Some(path) => {
            path
        },
        None => {
            &args.origin
        },
    };
    
    /*
    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(true)
        .open(write_path)
        .expect("Unable to open or create file");
    */
    //write!(&mut file, "{}", replaced).expect("Unable to write");
    println!("{}",replaced);
}
