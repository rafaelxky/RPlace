use core::panic;
use std::{collections::HashMap, hash::Hash};

use crate::parser::Node;

pub struct Writer {
    nodes: Vec<Node>,
}
impl Writer {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }
    // I need to save " " and \n data
    pub fn replace(&self) -> String {
        let mut text = String::new();
        let mut def_map: HashMap<String, Node> = HashMap::new();

        self.nodes.iter().for_each(|node| match node {
            Node::DEF { name, body } => {
                def_map.insert(name.to_string(), node.clone());
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
            Node::PLACE { name, args } => {
                let mut args_map: HashMap<String,String> = HashMap::new();

                args.iter().for_each(|arg|{
                    println!("Arg to replace, {}, {}", arg.0, arg.1);
                    args_map.insert(arg.0.clone(), arg.1.clone());
                });

                match def_map.get(name) {
                Some(val) => match val {
                    Node::DEF { name:_, body } => {
                        match body.as_ref() {
                            Node::BODY { data } => {
                                data.iter().for_each(|n|{
                                    match n {
                                        Node::DATA { data } => {
                                            text.push_str(data);
                                        },
                                        Node::VARTEMPLATE { name } => {
                                            let replacement = match args_map.get(name) {
                                                Some(val) => {
                                                    val
                                                },
                                                None => {
                                                    panic!("No value specified for \"{}\"!",name)
                                                }
                                            };
                                            text.push_str(replacement);
                                        }
                                        _ => panic!("Body should only have data or var def, instead found {:?}", n)
                                    }
                                });
                            },
                            _ => panic!("Internal error, def has a node wich is not of type body! {:?}", body)
                        }
                    },
                    _ => panic!("Internal error, wrong insertion in map! {:?}", val)
                },
                None => {
                    panic!("No such template named {}", name);
                }
            }},
            _ => (),
        });
        println!("finished in writer");
        text
    }
}
