use std::{
    collections::HashMap,str
};
use rayon::{ prelude::*};
use rayon::iter::IntoParallelRefIterator;

use crate::derive::deriver::Deriver;
use crate::structs::*;
use crate::writer::writer_structs::{Derive, ResValue, WriterResult};
use crate::{data_stream::get_data_stream};

use crate::{
    error_handler::handle_error,
    lexer::Lexer,
    parser::{Parser},
};

pub struct Writer {
    nodes: Vec<Node>,
    file_path: String,
}
impl Writer {
    pub fn new(nodes: ParsingResult) -> Self {
        Self {
            nodes: nodes.nodes,
            file_path: nodes.file_path,
        }
    }

    fn handle_import(&self, data: String, path: String) -> (String, Vec<Node>) {
        let mut imports: Vec<Node> = Vec::new();
                let lexer = Lexer::new(path.clone(), data);
                let parser = Parser::new(lexer.parse());
                let nodes = parser.parse();
                nodes.nodes.iter().for_each(|n| {
                    if let Node::DEF {
                        name:_,
                        body: _,
                        line: _,
                        conditions: _,
                        defaults: _,
                    } = n
                    {
                        imports.push(n.clone());
                    }
                });
                (path, imports)
    }

    fn initial_sweap(&mut self, def_map: &mut HashMap<String, Vec<Node>>) {
        // initial sweap
        let mut to_import: Vec<(String,usize)> = Vec::new();

        for node in &self.nodes {
            match node {
            Node::DEF {
                name,
                body: _,
                line: _,
                conditions: _,
                defaults:_,
            } => {
                self.handle_def(&mut *def_map, &node, &name);
            },
            Node::INCLUDE { path, line } => {
               to_import.push((path.clone(),*line));
            }
            _ => (),
        }};

        let imports: Vec<Vec<(String,Vec<Node>)>> = to_import.par_iter().map(|(path, _line)|{
            let (mut stream, _) = get_data_stream(path);
            let mut imp = Vec::new();
            loop {
                let data = stream.next();
                if data.is_none() {
                    break;
                }
                let (data, path) = data.unwrap();
                imp.push(self.handle_import(data,path));
            }
            imp
        }).collect();

        for imports_inner in imports{
        for (_, import) in imports_inner {
            for node in import {
                match &node {
                    Node::DEF { conditions:_, defaults:_, name, body:_, line:_ } => {
                        def_map.entry(name.clone()).or_insert_with(Vec::new).push(node.clone());
                    },
                    _ => ()
                }
            }
        }
    }
        
    }

    fn handle_def(&self, def_map: &mut HashMap<String, Vec<Node>>, node: &Node, name: &String) {
        let name = name.to_string();
        def_map
            .entry(name)
            .or_insert_with(Vec::new)
            .push(node.clone());
    }

    /// raw body text just takes the values and pushes it to buffer
    fn handle_raw_text(text: &mut String, data: &Vec<Node>){
         data.iter().for_each(|n| match n {
                        Node::DATA { data, line:_ } => {
                            text.push_str(data);
                        }
                        Node::VARTEMPLATE { name } => {
                            text.push_str(&format!("$#{}", name));
                        }
                        _ => (),
                    });
    }

    pub fn replace(mut self) -> WriterResult {
        let mut result = WriterResult::new();

        let mut text = String::new();
        let mut def_map = HashMap::new();
        self.initial_sweap(&mut def_map);

        let nodes = &self.nodes;
        for node in nodes {
            match node {
                Node::BODY { data, line:_ } => {
                    Self::handle_raw_text(&mut text, data);
                },
                Node::DATA { data, line:_ } => {
                    text.push_str(&data);
                },
                Node::PLACE { name, args, line } => {
                    let mut args_map: HashMap<String, ResValue> = HashMap::new();
                    let (inner_defs,result_inner) =
                        self.handle_place(&mut text, &mut def_map, name, args, line, &mut args_map);
                    result.append(result_inner);
                    if matches!(inner_defs, Some(_)) {
                        inner_defs.unwrap().iter().for_each(|def| {
                            if let Node::DEF {
                                conditions: _,
                                defaults: _,
                                name,
                                body: _,
                                line: _,
                            } = def
                            {
                                self.handle_def(&mut def_map, def, name);
                            }
                        });
                    }
                },
                Node::CREATE { path, content } => {
                    let result_inner = self.handle_create(path, content, &def_map);
                    result.append(result_inner);
                },
                Node::DERIVE { path, val } => {
                    result.derives.push(Derive { path: path.to_string(), vals: val.clone() });
                },
                _ => (),
            }
        }
        result.push_elements(text, self.file_path);
        result
    }

    fn handle_create(&self, path: &str, body: &Option<Box<Node>>, def_map: &HashMap<String, Vec<Node>>) -> WriterResult {
        let path = path.to_string();
        let mut result = WriterResult::new();

        match body {
            Some(node) => {
                match &**node {
                    // body -> place
                    Node::BODY { data, line } => {
                        data.iter().for_each(|place|{
                            match place {
                                Node::PLACE { name, args, line } => {
                                    let mut text = String::new();
                                    let mut args_map: HashMap<String, ResValue> = HashMap::new();
                                    self.handle_place(&mut text, def_map, name, args, line, &mut args_map);
                                    result.push_elements(text, path.clone());
                                },
                                _ => {
                                    panic!("Expected only place inside of create instead found {:?} in line {}", place, line);
                                }
                            }
                        });
                        return result;
                    },
                    _ => {
                        panic!("Internal error, no body found inside create, found instead {:?}",node);
                    }
                }
            },
            None => {
                result.push_elements("".to_string(), path);
                result
            },
        }
    }

    fn handle_def_body(&self, data: &Vec<Node>, text: &mut String, args_map: &mut HashMap<String, ResValue>,def_queue: &mut Option<Vec<Node>>,  def_name: &String, line: &usize, def_map: &HashMap<String, Vec<Node>>, result: &mut WriterResult){
        // for each body node
        // go trough the body and handle the cases
        data.iter().for_each(|n| match n {
            // here is handled anything inside the def
            Node::DATA { data, line:_ } => {
                // just text
                text.push_str(data);
            },
            Node::MATCH { line, var_name, val } => {
                let match_line = line;
                let var_name = var_name;
                let var = args_map.get(var_name);
                let var_value = match var {
                    Some(val) => val,
                    None => panic!("todo error message"),
                };
                let matched = val.iter().find(|arm|{
                    arm.matches(var_value.value.to_string())
                });

                match matched {
                    Some(matched) => {
                        match &matched.body {
                            Node::BODY { data, line } => {
                                self.handle_def_body(data, text, args_map, def_queue, def_name, match_line, def_map, result);
                            }
                            _ => panic!("todo error message"),
                        }
                    },
                    None => {
                        panic!("todo error message")
                    }
                }

            }
            Node::VARTEMPLATE { name } => {
                // $#
                let replacement = match args_map.get(name) {
                    Some(val) => val,
                    None => {
                        handle_error(format!("No value specified for \"{}\" in template {}!", name,def_name), line.clone(), self.file_path.clone())
                    }
                };
                text.push_str(&replacement.value);
            },
            Node::RARROWVAR { name, default } => {
                // ->
                    let replacement = match args_map.get(name) {
                    Some(val) => val,
                    None => {
                        match default {
                            Some(default) => &ResValue { value: default.to_string()},
                            None => handle_error(format!("No value specified for \"{}\" in right arrow variable declaration!", name), line.clone(), self.file_path.clone())
                        }
                    }
                };
                // todo: handle options here
                text.push_str(&replacement.value);
            },
            Node::DEF { conditions: _, name: _, body:_, line: _ , defaults: _} => {
                if def_queue.is_none() {
                    *def_queue = Some(Vec::new());
                }
                def_queue.as_mut().unwrap().push(n.clone());
            }, 
                Node::PLACE { name, args, line } => {
                // def ident place ident ...
                if def_queue.is_none() {
                    *def_queue = Some(Vec::new());
                }
                let (result_inner, writer_result) = self.handle_place(text, &def_map, name, args, line, args_map);
                result.append(writer_result);
                if result_inner.is_some() {
                    def_queue.as_mut().unwrap().append(&mut result_inner.unwrap());
                }
            },
            Node::CREATE { path, content } =>{
                let result_inner = self.handle_create(path, content, def_map);
                result.append(result_inner);
            },
            _ => {
                handle_error(format!("Body should only have data or var def, instead found {:?}", n), line.clone(), self.file_path.clone())
            },
        });
    }

    // todo: inner create
    // todo: supported inner commands: def, derive
    fn handle_place(
        &self,
        text: &mut String,
        def_map: &HashMap<String, Vec<Node>>,
        name: &String,
        args: &Vec<(String, Value)>,
        line: &usize,
        args_map: &mut HashMap<String, ResValue>,
    ) 
    // def queue
    -> (Option<Vec<Node>>, WriterResult) {
        // maps variables to values
        args.iter().for_each(|arg| {
            // this is to avoid children overriding parent values
            if !args_map.contains_key(&arg.0.clone()) {
                match &arg.1.value_type {
                    &ValueType::Literal => { args_map.insert(arg.0.clone(), ResValue { value: arg.1.value.to_string() }); }
                    &ValueType::Var => {
                        let val = args_map.get(name);
                        match val {
                            Some(val) => {
                                 args_map.insert(arg.0.clone(), val.clone()); 
                            },
                            None => {
                                panic!("No value found for var type {:?} line ", name);
                            },
                        }
                    },
                }
            }
        });

        let mut def_queue: Option<Vec<Node>> = None;
        let mut result = WriterResult::new();

        match def_map.get(name) {
            // check if template exists
            Some(val) => {
                let mut has_conditions = false;
                let mut matched: Option<&Node> = None;

                // foreach def node in override
                for def in val {
                    if matched.is_some() && has_conditions {
                        break;
                    }
                    if let Node::DEF {
                        conditions,
                        defaults: _,
                        name: _,
                        body: _,
                        line: _,
                    } = def
                    {
                        // match conditions
                        match conditions {
                            Some(vec) => {
                                for eval in vec {
                                    let val = args_map.get(&eval.0);
                                    if val.is_none() {
                                        break;
                                    }
                                    //- def struct where lang = rust:
                                    //- place struct where lang = rust:
                                    if !eval.2.eval(&val.unwrap().value, &eval.1) {
                                        break;
                                    }
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

                // failed to find override
                if matched.is_none() {
                    handle_error(
                        format!("No available override for {:?}", name),
                        *line,
                        self.file_path.clone(),
                    )
                }
                // template to place
                match matched.unwrap() {
                    Node::DEF {
                        name: def_name,
                        body,
                        line,
                        conditions: _,
                        defaults,
                    } => {
                        if defaults.is_some() {
                            defaults.as_ref().unwrap().iter().for_each(|(var,val)|{
                                if !args_map.contains_key(var) {
                                    args_map.insert(var.clone(), ResValue { value: val.clone()});
                                }
                            });
                        }
                        match body.as_ref() {
                        // outer match is for stuff like def place, inner is for true body
                        Node::BODY { data, line } => {
                           self.handle_def_body(data, text, args_map, &mut def_queue, def_name, line, def_map, &mut result);
                        },
                        // def place
                        Node::PLACE { name, args, line } => {
                            self.handle_place(text, def_map, name, args, line, args_map);
                        },
                        // def derive
                        Node::DERIVE { path, val } => {
                            let result = Deriver::derive(&Derive { path: path.clone(), vals: val.clone() });
                            text.push_str(&result);
                        },
                        // def create
                        Node::CREATE { path, content } => {
                            self.handle_create(path, content, def_map);
                        },
                        _ => handle_error(
                            format!(
                                "Internal error, def has a node {:?} wich is not of type body or place",
                                body
                            ),
                            line.clone(),
                            self.file_path.clone(),
                        ),
                    }
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
            None => {
                let mut defs = String::new();
                def_map.iter().for_each(|(name,_)|{defs.push_str(&format!(" {} ", name));});
                handle_error(
                format!("No such template named {}, available{}", name, defs),
                *line,
                self.file_path.clone(),
            )
        },
        }
        return (def_queue,result);
    }
}
