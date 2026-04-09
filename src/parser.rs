use core::panic;
use std::process::exit;

use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Node {
    // - def template_1
    DEF { name: String, body: Box<Node> },
    // either data or var ($a)
    BODY { data: Vec<Node> },
    // def body
    DATA { data: String },
    // def variables
    VARTEMPLATE { name: String },
    PLACE { name: String },
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
        while self.can_pop() {
            let mut curr = self.pop();
            match curr {
                Token::MARK => {
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
                }
                _ => continue,
            }
        }
        println!("finished in parser");
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
        match self.peek() {
            Token::IDENT { str } => {
                self.pop();
                nodes.push(Node::PLACE { name: str });
            }
            _ => {
                panic!("{:?} cant go after DEF", self.peek())
            }
        }
        match self.peek() {
            Token::DD => {
                self.pop();
            }
            _ => {
                panic!("{:?} cant go after DEF name, forgot \":\" ?", self.peek())
            }
        }
    }
}
