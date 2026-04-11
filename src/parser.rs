use core::panic;
use std::process::exit;

use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Node {
    // - def template_1
    DEF {
        name: String,
        body: Box<Node>,
    },
    // either data or var ($a)
    BODY {
        data: Vec<Node>,
    },
    // def body
    DATA {
        data: String,
    },
    // def variables
    VARTEMPLATE {
        name: String,
    },
    PLACE {
        name: String,
        args: Vec<(String, String)>,
    },
}
pub struct Parser {
    tokens: Vec<Token>,
    ptr: usize,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, ptr: 0 }
    }
    fn peek(&self) -> Token {
        self.tokens[self.ptr].clone()
    }
    fn pop(&mut self) -> Token {
        self.ptr = self.ptr + 1;
        self.tokens[self.ptr - 1].clone()
    }
    fn can_pop(&self) -> bool {
        self.tokens.len() > self.ptr
    }

    pub fn parse(mut self) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::new();
        let mut body_str = String::new();
        while self.can_pop() {
            let curr = self.pop();
            match curr {
                Token::MARK => {
                    nodes.push(Node::DATA { data: body_str.to_string() });
                    body_str = String::new();
                    self.remove_spaces();
                    match self.peek() {
                        Token::DEF => {
                            self.pop();
                            self.handle_def(&mut nodes);
                        }
                        Token::PLACE => {
                            self.pop();
                            self.handle_place(&mut nodes);
                        }
                        _ => {
                            panic!("{:?} cannot go after a mark", self.peek());
                        }
                    }
                },
                Token::IDENT { str } => {
                    body_str.push_str(&str);
                },
                Token::COMMA => {
                    body_str.push(',');
                },
                Token::DD => {
                    body_str.push(':');
                },
                Token::LSRQBRACK => {
                    body_str.push('[');
                },
                Token::RSRQBRACK => {
                    body_str.push(']');
                },
                Token::SPACE => {
                    body_str.push(' ');
                },
                Token::EQUALS => {
                    body_str.push('=');
                },
                Token::WHERE => {
                    body_str.push_str("where");
                },
                Token::ENDEF => {
                    body_str.push_str("endef");
                },
                Token::NL => {
                    body_str.push('\n');
                }
                _ => continue,
            }
        }
        println!("finished in parser");
        nodes.push(Node::DATA { data: body_str.to_string() });
        nodes
    }

    fn handle_def(&mut self, nodes: &mut Vec<Node>) {
        //- def ...

        self.remove_spaces();

        let mut def_name = String::new();

        match self.peek() {
            Token::IDENT { str } => {
                self.pop();
                def_name = str;
            }
            _ => {
                panic!("found {:?} expected definition name", self.peek());
            }
        }

        self.remove_spaces();

        match self.peek() {
            Token::DD => {
                println!("Placed def {}", def_name);
                println!("curr {:?}", self.peek());
                self.pop();
            }
            _ => {
                panic!(
                    "{:?} is invalid in def declaration of name {}",
                    self.peek(),
                    def_name
                );
            }
        }

        // body handling
        let body = self.build_body();
        nodes.push(Node::DEF {
            name: def_name.to_string(),
            body: Box::new(body),
        });
    }

    fn remove_spaces(&mut self) {
        loop {
            match self.peek() {
                Token::SPACE => {
                    self.pop();
                }
                _ => return,
            }
        }
    }

    // ends at endef
    fn build_body(&mut self) -> Node {
        let mut body_str = String::new();
        let mut body: Vec<Node> = Vec::new();
        loop {
            match self.peek() {
                Token::IDENT { str } => {
                    body_str.push_str(&str);
                }
                Token::DD => {
                    body_str.push(':');
                }
                Token::SPACE => {
                    body_str.push(' ');
                }
                Token::NL => {
                    body_str.push('\n');
                }
                Token::COMMA => {
                    body_str.push(',');
                },
                Token::LSRQBRACK => {
                    body_str.push('[');
                },
                Token::RSRQBRACK => {
                    body_str.push(']');
                }
                Token::VAR => {
                    self.pop();
                    body.push(Node::DATA {
                        data: body_str.to_string(),
                    });
                    body_str = String::new();
                    self.remove_spaces();
                    match self.peek() {
                        Token::IDENT { str } => {
                            self.pop();
                            println!("var template: {}",str.to_string());
                            body.push(Node::VARTEMPLATE {
                                name: str.to_string(),
                            });
                            continue;
                        }
                        _ => {
                            panic!("expected IDENT found {:?}", self.peek())
                        }
                    }
                }
                Token::MARK => {
                    self.pop();
                    self.remove_spaces();
                    match self.peek() {
                        Token::ENDEF => {
                            self.pop();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                            });
                            self.remove_spaces();
                            match self.peek() {
                                Token::DD => {
                                    self.pop();
                                }
                                _ => {
                                    panic!("Endef found with no terminating \":\"");
                                }
                            }
                            break;
                        }
                        _ => {
                            panic!("{:?} is not a valid inner instruction", self.peek())
                        }
                    }
                }
                _ => {
                    panic!("unexpected inner token {:?}", self.peek());
                }
            }
            self.pop();
        }
        return Node::BODY { data: body };
    }

    fn handle_place(&mut self, nodes: &mut Vec<Node>) {
        self.remove_spaces();

        let place_id = match self.peek() {
            Token::IDENT { str } => {
                self.pop();
                str
            }
            _ => {
                panic!("{:?} cant go after DEF", self.peek())
            }
        };

        self.remove_spaces();
        match self.peek() {
            Token::DD => {
                self.pop();
                nodes.push(Node::PLACE {
                    name: place_id,
                    args: Vec::new(),
                });
                return;
            }
            Token::WHERE => {
                let mut args: Vec<(String, String)> = Vec::new();
                loop {
                self.pop();
                self.remove_spaces();
                match self.peek() {
                    Token::IDENT { str } => {
                        self.pop();
                        self.remove_spaces();
                        let from = str;
                        match self.peek() {
                            Token::EQUALS => {
                                self.pop();
                                self.remove_spaces();
                                match self.peek() {
                                    Token::IDENT { str } => {
                                        self.pop();
                                        println!("pushed arg in handle place, {} = {}", from, str);
                                        args.push((from, str));
                                        self.remove_spaces();
                                        match self.peek() {
                                            Token::COMMA => {
                                                self.pop();
                                            },
                                            Token::DD => {
                                                self.pop();
                                                nodes.push(Node::PLACE {
                                                    name: place_id.clone(),
                                                    args: args,
                                                });
                                                return;
                                            }
                                            _ => {
                                                panic!("expected , or : found {:?}", self.peek());
                                            }
                                        }
                                    }
                                    _ => {
                                        panic!(
                                            "expected argument value as ident, found {:?}",
                                            self.peek()
                                        );
                                    }
                                }
                            }
                            _ => {
                                panic!("expected = found {:?}", self.peek());
                            }
                        }
                    }
                    Token::DD => {
                        self.pop();
                        nodes.push(Node::PLACE {
                            name: place_id.clone(),
                            args: Vec::new(),
                        });
                        return;
                    }
                    _ => {
                        panic!("{:?} invalid after WHERE", self.peek());
                    }
                }
            }},
            _ => {
                panic!("{:?} cant go after DEF name, forgot \":\" ?", self.peek())
            }
        }
    }
}
