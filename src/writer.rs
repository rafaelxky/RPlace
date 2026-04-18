use core::panic;
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
};

use crate::{
    error_handler::handle_error,
    lexer::Lexer,
    parser::{Node, Parser, ParsingResult}, term::data_providers::TextProvider,
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
    pub fn replace(mut self) -> String {
        let mut text = String::new();
        let mut def_map: HashMap<String, Vec<Node>> = HashMap::new();

        self.nodes.iter().for_each(|node| match node {
            Node::DEF {
                name,
                body,
                line,
                conditions,
            } => {
                let mut name = name.to_string();
                def_map
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .push(node.clone());
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
                let lexer = Lexer::new(path.clone(), TextProvider::get_text(&path));
                let parser = Parser::new(lexer.parse());
                let nodes = parser.parse();
                nodes.nodes.iter().for_each(|n| {
                    if let Node::DEF {
                        name,
                        body: _,
                        line,
                        conditions,
                    } = n
                    {
                        def_map
                            .entry(name.clone())
                            .or_insert_with(Vec::new)
                            .push(n.clone());
                    }
                });
            }
            _ => (),
        });

        let nodes = &self.nodes;
        for node in nodes {
            match node {
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
                    text.push_str(&data);
                }
                Node::PLACE { name, args, line } => {
                    let mut args_map: HashMap<String, String> = HashMap::new();
                    self.handle_place(&mut text, &def_map, name, args, line, &mut args_map);
                }
                _ => (),
            }
        }
        text
    }

    fn handle_place(
        &self,
        text: &mut String,
        def_map: &HashMap<String, Vec<Node>>,
        name: &String,
        args: &Vec<(String, String)>,
        line: &usize,
        args_map: &mut HashMap<String, String>,
    ) {

        args.iter().for_each(|arg| {
            if !args_map.contains_key(&arg.0.clone()) {
                args_map.insert(arg.0.clone(), arg.1.clone());
            }
        });

        let def = def_map.get(name);
        println!("Got {} from hasmap", name);
        match def {
            Some(val) => {
                let mut has_conditions = false;
                let mut matched = None;

                // foreach def node in the 
                for def in val {
                    println!("iter {:?} ",args_map);
                    if matched.is_some() && has_conditions {
                        println!("match is some and conditions");
                        break;
                    }
                    if let Node::DEF {
                        conditions,
                        name: _,
                        body: _,
                        line:_,
                    } = def
                    {
                        match conditions {
                            Some(vec) => {
                                for eval in vec {
                                    let val = args_map.get(&eval.0);
                                    if val.is_none() {
                                        break;
                                    }
                                    //- def struct where lang = rust:
                                    //- place struct where lang = rust:
                                    if !eval.2.eval(&val.unwrap(), &eval.1) {
                                        println!("Eval {} and {} failed",&val.unwrap(), eval.1);
                                        break;
                                    }
                                    println!("Eval {} and {} success",&val.unwrap(), eval.1);
                                    matched = Some(def);
                                    has_conditions = true;
                                }
                            }
                            None => {
                                matched = Some(def);
                            }
                        }
                    }
                }

                if matched.is_none() {
                    handle_error(format!("No available override for {:?}",def), *line, self.file_path.clone())                    
                }
                match matched.unwrap() {
                    Node::DEF {
                        name: _,
                        body,
                        line,
                        conditions,
                    } => match body.as_ref() {
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
                        Node::PLACE { name, args, line } => {
                            // def ident place ident ...
                            self.handle_place(text, def_map, name, args, line, args_map);
                        }
                        _ => handle_error(
                            format!(
                                "Internal error, def has a node {:?} wich is not of type body",
                                body
                            ),
                            *line,
                            self.file_path.clone(),
                        ),
                    },
                    _ => handle_error(
                        format!(
                            "Internal error, wrong insertion in map! Inserted node of type {:?} expected DEF",
                            val
                        ),
                        *line,
                        self.file_path.clone(),
                    ),
                }
            }
            None => handle_error(
                format!("No such template named {}", name),
                *line,
                self.file_path.clone(),
            ),
        }
    }
}
