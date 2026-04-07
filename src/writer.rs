use std::{collections::HashMap, hash::Hash};

use crate::parser::Node;

pub struct Writer{
    nodes: Vec<Node>
}
impl Writer {
    pub fn new(nodes: Vec<Node>) -> Self{
        Self{nodes}
    }   
    // I need to save " " and \n data
    pub fn replace(&self,val: &[(&str, &str)]) -> String{
        let mut text = String::new();
        let mut map: HashMap<String, String> = HashMap::new();

        val.iter().for_each(|(var_name, val)|{
            map.insert(var_name.to_string(), val.to_string());
        });

        self.nodes.iter().for_each(|node|{
            match node {
                Node::DEF { name, body } => {
                    match body.as_ref() {
                        Node::BODY { data } => {
                            data.iter().for_each(|node|{
                                match node {
                                    Node::DATA { data } => {
                                        text.push_str(data);
                                        text.push(' ');
                                    },
                                    Node::VARTEMPLATE { name } => {
                                        if map.contains_key(name) {
                                            text.push_str(map.get(name).unwrap());
                                            text.push(' ');
                                        } else {

                                        }
                                    }
                                    _ => (),
                                }
                            });
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        });
        text
    }
}