use std::sync::{Arc, RwLock};
use std::{
    collections::HashMap,str
};
use rayon::{ prelude::*};
use rayon::iter::IntoParallelRefIterator;

use crate::config::config::{CompilerConfig};
use crate::derive::deriver::Deriver;
use crate::options::var_options::{VarOptionsMap};
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
    imports: Arc<RwLock<HashMap<String, ParsingResult>>>,
    file_config: FileConfig,
    project_src: String,
    output_src: String,
    compiler_config: Arc<CompilerConfig>,
    var_options: Arc<VarOptionsMap>,
}
impl Writer {
    pub fn new(
        nodes: ParsingResult, 
        project_src: String, 
        output_src: String, 
        compiler_config: Arc<CompilerConfig>, 
        var_options: Arc<VarOptionsMap>
    ) -> Self {
        Self {
            nodes: nodes.nodes,
            file_path: nodes.file_path,
            imports: Arc::new(RwLock::new(HashMap::new())),
            file_config: FileConfig::default(),
            project_src,
            output_src,
            compiler_config,
            var_options,
        }
    }
    pub fn new_with_imports(
        nodes: ParsingResult, 
        imports: Arc<RwLock<HashMap<String,ParsingResult>>>, 
        project_src: String, 
        output_src: String, 
        compiler_config: Arc<CompilerConfig>,
        var_options: Arc<VarOptionsMap>,
    ) -> Self{
        Self {
            nodes: nodes.nodes,
            file_path: nodes.file_path,
            imports: imports,
            file_config: FileConfig::default(),
            project_src,
            output_src,
            compiler_config,
            var_options,
        }
    }

    fn handle_import(&self, data: String, path: String) -> ParsingResult {
        {
            let import_lock = self.imports.read().unwrap();
            let maybe_import = import_lock.get(&path);

            match maybe_import {
                Some(result) => {
                    return result.clone();
                },
                None => (),
            }
        }
                let lexer = Lexer::new(path.clone(), data);
                let parser = Parser::new(lexer.parse(),self.project_src.clone(), self.output_src.clone());
                let nodes = parser.parse();
                let mut import_lock = self.imports.write().unwrap();
                import_lock.insert(nodes.file_path.clone(), nodes.clone());
                nodes
    }

    fn initial_sweap(&mut self, def_map: &mut HashMap<String, Vec<Node>>, to_parse: &mut Vec<String>) {
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
            },
            Node::SETVARIABLE { var, val } => {
                if var.len() < 1 {
                    panic!("todo message: var assignement cant be empty");
                }
                
                if var.len() == 1 {
                    match val {
                        Value::Literal { value, options:_ } => {
                            self.file_config.set_val(&var[0], value.clone());
                        },
                        Value::Var { value:_, options:_ } => panic!("todo msg"),
                        Value::Array { values: _, names: _ } => panic!("todo msg"),
                    }
                    continue;
                }

                panic!("todo message: var assignement cant be empty");
            },
            Node::PARSE { path } => {
                to_parse.push(path.clone());
            }
            _ => (),
        }};

        let imports: Vec<Vec<ParsingResult>> = to_import.par_iter().map(|(path, _line)|{
            let (mut stream, _) = get_data_stream(path);
            let mut imp = Vec::new();
            loop {
                let data = stream.next();
                if data.is_none() {
                    break;
                }
                let (data, path) = data.unwrap();
                if !self.compiler_config.allow_import {
                    continue;
                }
                imp.push(self.handle_import(data,path));
            }
            imp
        }).collect();

        for imports_inner in imports{
        for import in imports_inner {
            for node in import.nodes {
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
                        Node::VARTEMPLATE { val } => {
                            let name = &val.value;
                            text.push_str(&format!("$#{}", name));
                        }
                        _ => (),
                    });
    }

    pub fn get_paths(mut self) -> Vec<String>{
        let mut def_map = HashMap::new();
        let mut to_parse = Vec::new();
        self.initial_sweap(&mut def_map, &mut to_parse);
        return to_parse;
    }
    pub fn replace(mut self) -> (WriterResult, FileConfig) {
        let mut result = WriterResult::new();

        let mut text = String::new();
        let mut def_map = HashMap::new();
        let mut to_parse = Vec::new();
        self.initial_sweap(&mut def_map, &mut to_parse);
        result.set_to_parse(to_parse);

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
        (result, self.file_config)
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

    // this function is called to place the def body
    fn handle_place_body(
        &self, data: &Vec<Node>, 
        text: &mut String, 
        args_map: &mut HashMap<String, ResValue>,
        def_queue: &mut Option<Vec<Node>>,  
        def_name: &String, 
        line: &usize, 
        def_map: &HashMap<String, Vec<Node>>, 
        result: &mut WriterResult)
        {
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
                    None => panic!("todo error message found no arg to match"),
                };
                let matched = val.iter().find(|arm|{
                    match var_value {
                        ResValue::Val { value } => {
                            arm.matches(value.to_string())
                        },
                        ResValue::Array { array } => todo!(),
                    }
                });

                match matched {
                    Some(matched) => {
                        match &matched.body {
                            Node::BODY { data, line:_ } => {
                                self.handle_place_body(data, text, args_map, def_queue, def_name, match_line, def_map, result);
                            }
                            _ => panic!("todo error message expected body"),
                        }
                    },
                    None => {
                        panic!("todo error message found no match")
                    }
                }

            }
            Node::VARTEMPLATE { val } => {
                let name = &val.value;
                // $#var 
                let replacement = match args_map.get(name) {
                    Some(val) => val,
                    None => {
                        if val.optional {
                            &ResValue::new_val("".to_string())
                        } else {
                            //handle_error(format!("No value specified for \"{}\" in template {}!", name,def_name), line.clone(), self.file_path.clone())
                            panic!("no falue specified for {}, in template {}", name, def_name)
                        }
                    }
                };
                let opts = &val.options;
                let replaced = match opts {
                    Some(opts) => {
                        let mut curr = match replacement {
                            ResValue::Val { value } => value.to_string(),
                            ResValue::Array { array } => todo!(),
                        };
                        for opt in opts {
                            curr = self.var_options.exec_option(opt, curr);
                        }
                        curr
                    }
                    None => {
                        match replacement {
                            ResValue::Val { value } => value.to_string(),
                            ResValue::Array { array } => todo!(),
                        }
                    },
                };
                text.push_str(&replaced);
            },
            Node::RARROWVAR { name, default } => {
                // ->
                    let replacement = match args_map.get(name) {
                    Some(val) => val,
                    None => {
                        match default {
                            Some(default) => &ResValue::new_val(default.to_string()),
                            None => {
                                handle_error(format!("No value specified for \"{}\" in right arrow variable declaration!", name), line.clone(), self.file_path.clone())
                            }
                        }
                    }
                };
                // todo: handle options here arrow var
                match replacement {
                    ResValue::Val { value } => {
                        text.push_str(&value);
                    },
                    ResValue::Array { array } => todo!(),
                }
            },
            Node::DEF { conditions: _, name: _, body:_, line: _ , defaults: _} => {
                if def_queue.is_none() {
                    *def_queue = Some(Vec::new());
                }
                def_queue.as_mut().unwrap().push(n.clone());
            }, 
            Node::PLACE { name, args, line } => {
                // place inside body
                // ??? whats this def_queue bellow ?
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
            Node::PARSE { path } => {
                result.push_to_parse(path.to_string());
            }
            Node::FOR { vars, in_var, body } => {
                let vals = args_map.get(in_var);
                let values = match vals {
                    Some(ResValue::Array { array }) => {
                        array
                    },
                    Some(ResValue::Val { value }) => panic!("expected array todo message"),
                    None => panic!("todo message, array var not found for in_var {in_var}"),
                }.clone();
                self.handle_for(
                    vars,
                    values,
                    body,
                    text, 
                    args_map,
                    def_queue,
                    def_name,
                    line,
                    def_map,
                    result,
                );
            }
            _ => {
                handle_error(format!("Body should only have data or var def, instead found {:?}", n), line.clone(), self.file_path.clone())
            },
        });
    }

    fn handle_for(
        &self, 
        var_names: &Vec<String>, 
        var_values: Vec<Vec<ResValue>>,
        body: &Box<Node>, 
        text: &mut String, 
        args_map: &mut HashMap<String, ResValue>,
        def_queue: &mut Option<Vec<Node>>,
        def_name: &String,
        line: &usize,
        def_map: &HashMap<String, Vec<Node>>,
        result: &mut WriterResult,
    )
    {
        for val in var_values {
            let mut args_map = args_map.clone();
            let mut def_queue = def_queue.clone();
            
            for (name,val) in var_names.iter().zip(val) {
                args_map.insert(name.to_string(), val);
            }
            

             match &**body {
            Node::BODY { data, line:_ } => {
                    self.handle_place_body(data, text, &mut args_map, &mut def_queue, def_name, line, def_map, result);
            }
            _ => panic!("todo error message expected body"),
        }
        }
        
    }

    fn resolve_var(&self, var: &Var, value: &Value, args_map: &HashMap<String, ResValue>, name: &String) -> (String, ResValue){
        match value {
            Value::Literal{value,options: _} => {
                //args_map.insert(var.name.clone(), ResValue::new_val(value.to_string()));
                println!("name: {}, val: {:?}", var.name,value.to_string());
                return (var.name.clone(),ResValue::new_val(value.to_string()));
            }
            Value::Var{value, options: _} => {
                let val = args_map.get(value);
                match val {
                    Some(val) => {
                        //args_map.insert(var.name.clone(), val.clone());
                        println!("name: {}, val: {:?}", value,val);
                        return (value.clone(),val.clone());
                    },
                    None => {
                        panic!("No value found for var type {:?} line ", value);
                    },
                }
            },
            Value::Array { values, names: _ } => {
                // todo: resolve array values
                let resolved_array_values = values.iter().map(|v|{
                    return v.iter().map(|val|{
                        return self.resolve_var(var, val, args_map, name).1;
                    }).collect();
                }).collect();
                return (var.name.clone(), ResValue::new_array(resolved_array_values));
            },
        }
    }

    fn handle_place(
        &self,
        text: &mut String,
        def_map: &HashMap<String, Vec<Node>>,
        name: &String,
        args: &Vec<(Var, Value)>,
        line: &usize,
        args_map: &mut HashMap<String, ResValue>,
    ) 
    // def queue
    -> (Option<Vec<Node>>, WriterResult) {
        // maps variables to values
        args.iter().for_each(|arg| {
            // this is to avoid children overriding parent values
            // maybe the best is to clone the hashmap 
            if !args_map.contains_key(&arg.0.name.clone()) {
                // resolve variables
                let (name,val) = self.resolve_var(&arg.0, &arg.1, args_map, name);
                args_map.insert(name, val);
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
                                   
                                    //- def struct where lang = rust:
                                    //- place struct where lang = rust:
                                    let value = match val {
                                        Some(ResValue::Val { value }) => {
                                            value
                                        },
                                        Some(ResValue::Array { array }) => todo!(),
                                        None => break,
                                    };
                                    if !eval.2.eval(value, &eval.1) {
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
                                    args_map.insert(var.clone(), ResValue::new_val(val.clone()));
                                }
                            });
                        }
                        match body.as_ref() {
                        // outer match is for stuff like def place, inner is for true body
                        Node::BODY { data, line } => {
                           self.handle_place_body(data, text, args_map, &mut def_queue, def_name, line, def_map, &mut result);
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
