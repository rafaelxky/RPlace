use std::collections::HashMap;
use std::fs::OpenOptions;
use std::process::exit;
use std::sync::{Arc, RwLock};

use crate::config::config::CompilerConfig;
use crate::data_stream::{DataSouce, get_data_stream};
use crate::lexer::Lexer;
use crate::lua::lua_call_map::LuaCallMap;
use crate::options::var_options::VarOptionsMap;
use crate::output_stream::OutputWriter;
use crate::package_manager::package_structs::PackageData;
use crate::parser::Parser;
use crate::structs::FileConfig;
use crate::term::terminal_handler::ParseArgs;
use crate::writer::writer::Writer;
use crate::writer::writer_structs::WriterResult;

pub fn parse_get_all_paths(data: PackageData, config: CompilerConfig) -> Vec<String>{
    let project_src = data.package.root;
    let output_src = "".to_string();
    let (mut stream, _origin) = get_data_stream(&project_src);

    let imports = Arc::new(RwLock::new(HashMap::new()));
    let config = Arc::new(config);
    let lua_map = LuaCallMap::load(config.clone());
    let var_options_map = Arc::new(VarOptionsMap::new(config.clone(), lua_map));
    let mut paths_outer: Vec<String> = vec![];

    loop {
        let data = stream.next();
        if data.is_none() {
            break;
        }
        let (data, path) = data.unwrap();
        let lexer = Lexer::new(path.clone(), data);
        let tokens = lexer.parse();
        let parser = Parser::new(tokens, project_src.clone(), output_src.to_string());
        let nodes = parser.parse();
        let path = nodes.file_path.clone();
        let writer = Writer::new_with_imports(
            nodes,
            imports.clone(),
            project_src.clone(),
            output_src.clone(),
            config.clone(),
            var_options_map.clone(),
        );
        let (mut to_parse, mut moded): (Vec<String>, Vec<String>) = writer.get_paths();
        stream.append(&mut to_parse);
        stream.append(&mut moded);
        paths_outer.push(path);
    }
    return paths_outer;
}

pub fn run_parse(args: ParseArgs, config: CompilerConfig) -> Vec<OutputWriter> {
    let (mut stream, origin) = get_data_stream(args.origin.as_ref().unwrap());
    let project_src = args.origin.unwrap();
    let output_src = match &args.target {
        Some(t) => t.clone(),
        None => project_src.clone(),
    };
    match origin {
        DataSouce::WEB => {
            if args.target.is_none() {
                eprintln!("No target file specified for web data souce");
                exit(1);
            }
        }
        DataSouce::FILE => (),
        DataSouce::Package => (),
    }

    // todo fix target path to create subfolders
    // todo make so that derive can create folders
    // fix imports check b.txt
    // fix import space between : and ident not working
    let imports = Arc::new(RwLock::new(HashMap::new()));
    let config = Arc::new(config);
    let lua_map = LuaCallMap::load(config.clone());
    let var_options_map = Arc::new(VarOptionsMap::new(config.clone(), lua_map));
    let mut to_write = vec![];
    loop {
        let data = stream.next();
        if data.is_none() {
            break;
        }
        let (data, path) = data.unwrap();
        let lexer = Lexer::new(path.clone(), data);
        let tokens = lexer.parse();
        let parser = Parser::new(tokens, project_src.clone(), output_src.clone());
        let nodes = parser.parse();
        if args.stops_at_parser {
            println!("{:#?}", nodes);
        }
        let writer = Writer::new_with_imports(
            nodes,
            imports.clone(),
            project_src.clone(),
            output_src.clone(),
            config.clone(),
            var_options_map.clone(),
        );
        let (mut replaced, config): (WriterResult, FileConfig) = writer.replace();
        stream.append(&mut replaced.to_parse);

        if args.stops_at_parser {
            continue;
        }
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
        to_write.push(output);
    }

    return to_write;
}
