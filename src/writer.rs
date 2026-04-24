use std::{
    collections::HashMap, path::{Path, PathBuf}, process::{exit}, str
};

use crate::{
    error_handler::handle_error,
    lexer::Lexer,
    parser::{Node, Parser, ParsingResult, Value, VarOptions},
    term::data_providers::TextProvider,
};
#[derive(Debug, Clone)]
pub struct ResValue{
    value:String,
    options: Option<Vec<VarOptions>>,
}
pub enum FileWriteOptions{
    Override,
    Append,
}
pub struct FileData{
    pub data: String,
    pub path: String,
    pub options: Option<Vec<FileWriteOptions>>
}
pub struct Derive{
    pub path: String, 
    pub val: Vec<(String,Value)>,
}
pub struct WriterResult {
    pub file_data: Vec<FileData>,
    pub derives: Vec<Derive>,
}
impl WriterResult {
    pub fn new() -> Self{
        Self { file_data: Vec::new(), derives: Vec::new()}
    }
    pub fn append(&mut self, mut data: WriterResult){
        self.file_data.append(&mut data.file_data);
        self.derives.append(&mut data.derives);
    }
    pub fn push_elements(&mut self, data: String, path: String){
        self.file_data.push(FileData { data, path, options: None });
    }
}
impl IntoIterator for WriterResult {
    type Item = FileData;
    type IntoIter = std::vec::IntoIter<FileData>;

    fn into_iter(self) -> Self::IntoIter {
        self.file_data.into_iter()
    }
}
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

    fn initial_sweap(&mut self, def_map: &mut HashMap<String, Vec<Node>>) {
        // initial sweap
        self.nodes.iter().for_each(|node| match node {
            Node::DEF {
                name,
                body: _,
                line: _,
                conditions: _,
                defaults:_,
            } => {
                self.handle_def(def_map, &node, &name);
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
                let lexer = Lexer::new(path.clone(), TextProvider::get_text(&path).0);
                let parser = Parser::new(lexer.parse());
                let nodes = parser.parse();
                nodes.nodes.iter().for_each(|n| {
                    if let Node::DEF {
                        name,
                        body: _,
                        line: _,
                        conditions: _,
                        defaults: _,
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
    }

    fn handle_def(&self, def_map: &mut HashMap<String, Vec<Node>>, node: &Node, name: &String) {
        let name = name.to_string();
        def_map
            .entry(name)
            .or_insert_with(Vec::new)
            .push(node.clone());
    }

    pub fn replace(mut self) -> WriterResult {
        let mut result = WriterResult::new();

        let mut text = String::new();
        let mut def_map: HashMap<String, Vec<Node>> = HashMap::new();

        self.initial_sweap(&mut def_map);

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
                },
                Node::DATA { data } => {
                    text.push_str(&data);
                },
                Node::PLACE { name, args, line } => {
                    let mut args_map: HashMap<String, ResValue> = HashMap::new();
                    let inner_defs =
                        self.handle_place(&mut text, &mut def_map, name, args, line, &mut args_map);

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
                    result.derives.push(Derive { path: path.to_string(), val: val.clone() });
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
                    Node::BODY { data } => {
                        data.iter().for_each(|place|{
                            match place {
                                Node::PLACE { name, args, line } => {
                                    let mut text = String::new();
                                    let mut args_map: HashMap<String, ResValue> = HashMap::new();
                                    self.handle_place(&mut text, def_map, name, args, line, &mut args_map);
                                    result.push_elements(text, path.clone());
                                },
                                _ => {
                                    eprintln!("Expected only place inside of create instead found {:?}", place);
                                    exit(1);
                                }
                            }
                        });
                        return result;
                    },
                    _ => {
                        eprintln!("Internal error, no body found inside create, found instead {:?}",node);
                        exit(1);
                    }
                }
            },
            None => {
                result.push_elements("".to_string(), path);
                result
            },
        }
    }

    // todo: inner create
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
    -> Option<Vec<Node>> {
        // maps variables to values
        args.iter().for_each(|arg| {
            // this is to avoid children overriding parent values
            if !args_map.contains_key(&arg.0.clone()) {
                match &arg.1 {
                    Value::Literal{value, options} => { args_map.insert(arg.0.clone(), ResValue { value: value.to_string(), options:options.clone() }); }
                    Value::Var{name,options} => {
                        let val = args_map.get(name);
                        match val {
                            Some(val) => {
                                 args_map.insert(arg.0.clone(), val.clone()); 
                            },
                            None => {
                                panic!("No value found for var type {:?}", name);
                            },
                        }
                    },
                }
            }
        });

        let mut def_queue: Option<Vec<Node>> = None;

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
                match matched.unwrap() {
                    Node::DEF {
                        name: _,
                        body,
                        line,
                        conditions: _,
                        defaults,
                    } => {
                        if defaults.is_some() {
                            defaults.as_ref().unwrap().iter().for_each(|(var,val)|{
                                if !args_map.contains_key(var) {
                                    args_map.insert(var.clone(), ResValue { value: val.clone(), options: None });
                                }
                            });
                        }
                        match body.as_ref() {
                        Node::BODY { data } => {
                            // for each body node
                            // go trough the body and handle the cases
                            data.iter().for_each(|n| match n {
                                    // here is handled anything inside the def
                                    Node::DATA { data } => {
                                        // just text
                                        text.push_str(data);
                                    },
                                    Node::VARTEMPLATE { name } => {
                                        // $#
                                        let replacement = match args_map.get(name) {
                                            Some(val) => val,
                                            None => {
                                                handle_error(format!("No value specified for \"{}\"!", name), line.clone(), self.file_path.clone())
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
                                                    Some(default) => &ResValue { value: default.to_string(), options: None },
                                                    None => handle_error(format!("No value specified for \"{}\" in right arrow variable declaration!", name), line.clone(), self.file_path.clone())
                                                }
                                            }
                                        };
                                        // todo: handle options here
                                        text.push_str(&replacement.value);
                                    },
                                    Node::DEF { conditions: _, name: _, body:_, line: _ , defaults: _} => {
                                        if def_queue.is_none() {
                                            def_queue = Some(Vec::new());
                                        }
                                        def_queue.as_mut().unwrap().push(n.clone());
                                    }, 
                                        Node::PLACE { name, args, line } => {
                                        // def ident place ident ...
                                        if def_queue.is_none() {
                                            def_queue = Some(Vec::new());
                                        }
                                        let result = self.handle_place(text, &def_map, name, args, line, args_map);
                                        if result.is_some() {
                                            def_queue.as_mut().unwrap().append(&mut result.unwrap());
                                        }
                                    }
                                    _ => {
                                        handle_error(format!("Body should only have data or var def, instead found {:?}", n), line.clone(), self.file_path.clone())
                                },
                                });
                        },
                        Node::PLACE { name, args, line } => {
                            self.handle_place(text, def_map, name, args, line, args_map);
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
            None => handle_error(
                format!("No such template named {}", name),
                *line,
                self.file_path.clone(),
            ),
        }
        return def_queue;
    }
}
