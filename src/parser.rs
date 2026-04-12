use core::panic;
use std::process::exit;

use crate::{
    error_handler::handle_error,
    lexer::{Token, TokenResult},
};

#[derive(Debug, Clone)]
pub enum Node {
    // - def template_1
    DEF {
        name: String,
        body: Box<Node>,
        line: usize,
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
        name: String
    },
    RARROWVAR {
        name: String,
        default: Option<String>,
    },
    PLACE {
        name: String,
        args: Vec<(String, String)>,
        line: usize,
    },
    INCLUDE {
        path: String,
        line: usize,
    },
}
pub struct ParsingResult {
    pub nodes: Vec<Node>,
    pub file_path: String,
}
pub struct Parser {
    tokens: Vec<Token>,
    ptr: usize,
    line: usize,
    file_path: String,
}
impl Parser {
    pub fn new(tokens: TokenResult) -> Self {
        Self {
            tokens: tokens.tokens,
            ptr: 0,
            line: 0,
            file_path: tokens.file_path,
        }
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

    pub fn parse(mut self) -> ParsingResult {
        let mut nodes: Vec<Node> = Vec::new();
        let mut body_str = String::new();
        while self.can_pop() {
            let curr = self.pop();
            match curr {
                Token::MARK { kind } => {
                    nodes.push(Node::DATA {
                        data: body_str.to_string(),
                    });
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
                        Token::INCLUDE => {
                            self.pop();
                            self.handle_include(&mut nodes);
                        }
                        _ => {
                            panic!(
                                "{:?} cannot go after an initial mark in line {}, did you forget a mark?",
                                self.peek(),
                                self.line
                            );
                        }
                    }
                }
                Token::INCLUDE => {
                    body_str.push_str("include");
                }
                Token::IDENT { str } => {
                    body_str.push_str(&str);
                }
                Token::COMMA => {
                    body_str.push(',');
                }
                Token::DQUOTE => {
                    body_str.push('"');
                }
                Token::DD => {
                    body_str.push(':');
                }
                Token::LSRQBRACK => {
                    body_str.push('[');
                }
                Token::RSRQBRACK => {
                    body_str.push(']');
                }
                Token::SPACE => {
                    body_str.push(' ');
                }
                Token::EQUALS => {
                    body_str.push('=');
                }
                Token::WHERE => {
                    body_str.push_str("where");
                }
                Token::ENDEF => {
                    body_str.push_str("endef");
                }
                Token::NL => {
                    self.line = self.line + 1;
                    body_str.push('\n');
                }
                Token::DEF => body_str.push_str("def"),
                Token::VAR => body_str.push_str("var"),
                Token::EOF => (),
                Token::PLACE => body_str.push_str("place"),
                Token::RARROW => body_str.push_str("->"),
            }
        }
        println!("finished in parser");
        nodes.push(Node::DATA {
            data: body_str.to_string(),
        });
        ParsingResult {
            nodes,
            file_path: self.file_path,
        }
    }

    fn handle_include(&mut self, nodes: &mut Vec<Node>) {
        let mut path = String::new();

        self.remove_spaces();

        loop {
            match self.peek() {
                Token::IDENT { str } => {
                    path.push_str(&str);
                }
                Token::DD => {
                    self.pop();
                    nodes.push(Node::INCLUDE {
                        path: path.clone(),
                        line: self.line,
                    });
                    return;
                }
                Token::INCLUDE => {
                    path.push_str("include");
                }
                Token::WHERE => {
                    path.push_str("where");
                }
                _ => {
                    panic!(
                        "Unexpected token in include declaration {:?} in line {}",
                        self.peek(),
                        self.line
                    );
                }
            }
            self.pop();
        }
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
                panic!(
                    "found {:?} expected definition name in line {}",
                    self.peek(),
                    self.line
                );
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
                    "{:?} is invalid in def declaration of name {} in line {}",
                    self.peek(),
                    def_name,
                    self.line
                );
            }
        }

        // body handling
        let body = self.build_body();
        nodes.push(Node::DEF {
            name: def_name.to_string(),
            body: Box::new(body),
            line: self.line,
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
                Token::INCLUDE => {
                    body_str.push_str("include");
                }
                Token::DD => {
                    body_str.push(':');
                }
                Token::RARROW => {
                    body_str.push_str("->");
                }
                Token::SPACE => {
                    body_str.push(' ');
                }
                Token::NL => {
                    self.line = self.line + 1;
                    body_str.push('\n');
                }
                Token::COMMA => {
                    body_str.push(',');
                }
                Token::LSRQBRACK => {
                    body_str.push('[');
                }
                Token::RSRQBRACK => {
                    body_str.push(']');
                }
                Token::DQUOTE => {
                    body_str.push('"');
                }
                // regular var declaration
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
                            println!("var template: {}", str.to_string());
                            body.push(Node::VARTEMPLATE {
                                name: str.to_string(),
                            });
                            continue;
                        }
                        _ => {
                            panic!(
                                "expected IDENT found {:?} in line {}",
                                self.peek(),
                                self.line
                            )
                        }
                    }
                }
                Token::MARK { kind } => {
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
                                    panic!(
                                        "Endef found with no terminating \":\" or \",\" in line {}",
                                        self.line
                                    );
                                }
                            }
                            break;
                        }
                        Token::VAR => {
                            self.pop();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                            });
                            body_str = String::new();
                            match self.peek() {
                                /*- #$ident -> -*/
                                Token::IDENT { str } => {
                                    let name = str;
                                    self.pop();
                                    let spaces = self.collect_spaces();
                                    match self.peek() {
                                        Token::RARROW => {
                                            self.pop();
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::MARK { kind } => {
                                                    self.pop();
                                                    self.remove_spaces();
                                                    self.pop();
                                                    match self.pop() {
                                                        Token::IDENT{str} => {
                                                            body.push(Node::RARROWVAR { name, default: Some(str.clone())});
                                                        },
                                                        _ => {
                                                            body.push(Node::RARROWVAR { name, default: None });
                                                        }
                                                    }
                                                    continue;
                                                }
                                                _ => handle_error(
                                                    format!(
                                                        "Expected marker at the end of right arrow variable declaration, found {:?}",
                                                        self.peek()
                                                    ),
                                                    self.line,
                                                    self.file_path.clone(),
                                                ),
                                            }
                                        }
                                        _ => handle_error(
                                            format!("Malformed marked variable inside body"),
                                            self.line,
                                            self.file_path.clone(),
                                        ),
                                    }
                                }
                                _ => handle_error(
                                    format!(
                                        "Found invalid token {:?} in arrow var declaration in def body",
                                        self.peek()
                                    ),
                                    self.line,
                                    self.file_path.clone(),
                                ),
                            }
                        }
                        _ => {
                            panic!(
                                "{:?} is not a valid inner instruction in line {}",
                                self.peek(),
                                self.line
                            )
                        }
                    }
                }
                Token::DEF => body_str.push_str("def"),
                Token::ENDEF => body_str.push_str("endef"),
                Token::EOF => (),
                Token::PLACE => body_str.push_str("place"),
                Token::WHERE => body_str.push_str("where"),
                Token::EQUALS => body_str.push('='),
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
                panic!("{:?} cant go after DEF in line {}", self.peek(), self.line)
            }
        };

        self.remove_spaces();
        match self.peek() {
            Token::DD => {
                self.pop();
                nodes.push(Node::PLACE {
                    name: place_id,
                    args: Vec::new(),
                    line: self.line,
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
                                        // ident = ident -> variable assignement
                                        Token::IDENT { str } => {
                                            self.pop();
                                            println!(
                                                "pushed arg in handle place, {} = {}",
                                                from, str
                                            );
                                            args.push((from, str));
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::COMMA => {
                                                    self.pop();
                                                }
                                                Token::DD => {
                                                    self.pop();
                                                    nodes.push(Node::PLACE {
                                                        name: place_id.clone(),
                                                        args: args,
                                                        line: self.line,
                                                    });
                                                    return;
                                                }
                                                _ => {
                                                    panic!(
                                                        "expected , or : found {:?} in line {}",
                                                        self.peek(),
                                                        self.line
                                                    );
                                                }
                                            }
                                        }
                                        // ident = "ident" -> quotation handling for multiline values
                                        Token::DQUOTE => {
                                            self.pop();
                                            let mut arg_str = String::new();
                                            let mut has_new_line = false;

                                            loop {
                                                if !self.can_pop() {
                                                    panic!(
                                                        "unexpected EOF in \"quotation\" variable in line {}",
                                                        self.line
                                                    )
                                                }
                                                match self.peek() {
                                                    Token::IDENT { str } => {
                                                        arg_str.push_str(&str);
                                                    }
                                                    Token::RARROW => {
                                                        arg_str.push_str("->");
                                                    }
                                                    Token::INCLUDE => {
                                                        arg_str.push_str("include");
                                                    }
                                                    Token::COMMA => {
                                                        arg_str.push(',');
                                                    }
                                                    Token::DD => {
                                                        arg_str.push(':');
                                                    }
                                                    Token::DEF => {
                                                        arg_str.push_str("def");
                                                    }
                                                    Token::ENDEF => {
                                                        arg_str.push_str("endef");
                                                    }
                                                    Token::EQUALS => {
                                                        arg_str.push('=');
                                                    }
                                                    Token::LSRQBRACK => {
                                                        arg_str.push('[');
                                                    }
                                                    Token::RSRQBRACK => {
                                                        arg_str.push(']');
                                                    }
                                                    Token::WHERE => {
                                                        arg_str.push_str("where");
                                                    }
                                                    Token::NL => {
                                                        self.line = self.line + 1;
                                                        arg_str.push('\n');
                                                        has_new_line = true;
                                                    }
                                                    Token::DQUOTE => {
                                                        if has_new_line {
                                                            arg_str.push('"');
                                                        } else {
                                                            self.pop();
                                                            break;
                                                        }
                                                    }
                                                    Token::VAR => {
                                                        arg_str.push_str("#$");
                                                    }
                                                    Token::MARK { kind } => {
                                                        self.pop();
                                                        if !has_new_line {
                                                            arg_str.push_str(&kind);
                                                        } else {
                                                            // if value has a newline after the first ", then ends at mark + "
                                                            let mut spaces = String::new();
                                                            let mut ends = false;
                                                            loop {
                                                                match self.peek() {
                                                                    Token::SPACE => {
                                                                        self.pop();
                                                                        spaces.push(' ');
                                                                    }
                                                                    Token::DQUOTE => {
                                                                        println!("dquote");
                                                                        self.pop();
                                                                        ends = true;
                                                                        break;
                                                                    }
                                                                    _ => {
                                                                        println!(
                                                                            "No break {:?}",
                                                                            self.peek()
                                                                        );
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            if ends {
                                                                println!("brake");
                                                                break;
                                                            } else {
                                                                arg_str.push_str(&spaces);
                                                            }
                                                            // if has " after mark
                                                        }
                                                    }
                                                    Token::EOF => (),
                                                    Token::PLACE => {
                                                        arg_str.push_str("place");
                                                    }
                                                    Token::SPACE => arg_str.push(' '),
                                                }
                                                self.pop();
                                            }
                                            args.push((from, arg_str));
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::DD => {
                                                    self.pop();
                                                    nodes.push(Node::PLACE {
                                                        name: place_id.clone(),
                                                        args: args,
                                                        line: self.line,
                                                    });
                                                    return;
                                                }
                                                Token::COMMA => {
                                                    self.pop();
                                                }
                                                _ => {
                                                    panic!(
                                                        "expected , or : found {:?} in line {}",
                                                        self.peek(),
                                                        self.line
                                                    );
                                                }
                                            }
                                        }
                                        _ => {
                                            panic!(
                                                "expected argument value as ident, found {:?} in line {}",
                                                self.peek(),
                                                self.line
                                            );
                                        }
                                    }
                                }
                                _ => {
                                    panic!(
                                        "expected = found {:?} in line {}",
                                        self.peek(),
                                        self.line
                                    );
                                }
                            }
                        }
                        Token::DD => {
                            self.pop();
                            nodes.push(Node::PLACE {
                                name: place_id.clone(),
                                args: Vec::new(),
                                line: self.line,
                            });
                            return;
                        }
                        _ => {
                            panic!(
                                "{:?} invalid after WHERE in line {}",
                                self.peek(),
                                self.line
                            );
                        }
                    }
                }
            }
            _ => {
                panic!(
                    "{:?} cant go after DEF name in line {}, forgot \":\" or \",\" ?",
                    self.peek(),
                    self.line
                )
            }
        }
    }

    fn collect_spaces(&mut self) -> (String, bool) {
        let mut spaces = String::new();
        let mut ends = false;
        loop {
            match self.peek() {
                Token::SPACE => {
                    self.pop();
                    spaces.push(' ');
                }
                Token::DQUOTE => {
                    self.pop();
                    ends = true;
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        (spaces, ends)
    }
}
