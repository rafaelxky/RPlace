use core::panic;
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
};

use crate::{
    error_handler::handle_error,
    lexer::Lexer,
    parser::{Node, Parser, ParsingResult},
};

pub struct Writer {
    nodes: Vec<Node>,
    line: usize,
    file_path: String,
}
impl Writer {
    pub fn new(nodes: ParsingResult) -> Self {
        Self {
            nodes: nodes.nodes,
            line: 0,
            file_path: nodes.file_path,
        }
    }
    // I need to save " " and \n data
    pub fn replace(&self) -> String {
        let mut text = String::new();
        let mut def_map: HashMap<String, Node> = HashMap::new();

        self.nodes.iter().for_each(|node| match node {
            Node::DEF { name, body, line } => {
                def_map.insert(name.to_string(), node.clone());
            }
            Node::INCLUDE { path, line } => {
                let mut path = path.clone();
                if let Some(stripped) = path.strip_prefix("~") {
                    let stripped = stripped.strip_prefix("/").unwrap_or(stripped);
                    path = PathBuf::from(std::env::var("HOME").unwrap())
                        .join(".rplace")
                        .join(stripped)
                        .to_string_lossy()
                        .to_string();
                }
                if !Path::new(&path).exists() {
                    handle_error("Couldnt find import", *line, &self.file_path);
                }
                let lexer = Lexer::new(path);
                let parser = Parser::new(lexer.parse());
                let nodes = parser.parse();
                nodes.nodes.iter().for_each(|n| {
                    if let Node::DEF {
                        name,
                        body: _,
                        line,
                    } = n
                    {
                        def_map.insert(name.clone(), n.clone());
                    }
                });
            }

            _ => (),
        });

        self.nodes.iter().for_each(|node| match node {
            Node::BODY { data } => {
                data.iter().for_each(|n| match n {
                    Node::DATA { data } => {
                        text.push_str(data);
                    }
                    Node::VARTEMPLATE { name } => {
                        text.push_str(&format!("$#{}", name));
                    }
                    _ => (),
                });
            }
            Node::DATA { data } => {
                text.push_str(data);
            }
            Node::PLACE { name, args, line } => {
                let mut args_map: HashMap<String, String> = HashMap::new();

                args.iter().for_each(|arg| {
                    args_map.insert(arg.0.clone(), arg.1.clone());
                });

                match def_map.get(name) {
                    Some(val) => match val {
                        Node::DEF { name: _, body, line } => match body.as_ref() {
                            Node::BODY { data } => {
                                // for each body node
                                data.iter().for_each(|n| match n {
                                    Node::DATA { data } => {
                                        text.push_str(data);
                                    }
                                    Node::VARTEMPLATE { name } => {
                                        let replacement = match args_map.get(name) {
                                            Some(val) => val,
                                            None => {
                                                handle_error(format!("No value specified for \"{}\"!", name), *line, self.file_path.clone())
                                            }
                                        };
                                        text.push_str(replacement);
                                    },
                                    Node::RARROWVAR { name, default } => {
                                         let replacement = match args_map.get(name) {
                                            Some(val) => val,
                                            None => {
                                                match default {
                                                    Some(default) => default,
                                                    None => handle_error(format!("No value specified for \"{}\" in right arrow variable declaration!", name), *line, self.file_path.clone())
                                                }
                                            }
                                        };
                                        text.push_str(replacement);
                                    }
                                    _ => {
                                        handle_error(format!("Body should only have data or var def, instead found {:?}", n), *line, self.file_path.clone())
                                },
                                });
                            }
                            _ => {
                            handle_error(format!("Internal error, def has a node {:?} wich is not of type body", body), *line, self.file_path.clone())
                        },
                        },
                        _ => {
                                handle_error(format!("Internal error, wrong insertion in map! Inserted node of type {:?} expected DEF", val), *line, self.file_path.clone())
                            }
                    },
                    None => {
                        handle_error(format!("No such template named {}", name), *line, self.file_path.clone())
                    }
                }
            }
            _ => (),
        });
        text
    }
}
