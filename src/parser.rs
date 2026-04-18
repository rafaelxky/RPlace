use core::panic;
use std::process::exit;

use crate::{
    error_handler::handle_error,
    lexer::{Token, TokenResult},
};

#[derive(Debug, Clone)]
pub enum Condition {
    EQUALS,
}
impl Condition {
    pub fn eval(&self, first: &str, sec: &str) -> bool {
        match self {
            Condition::EQUALS => {
                return first == sec;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    // - def template_1
    DEF {
        conditions: Option<Vec<(String, String, Condition)>>,
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
        name: String,
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
        self.parse_inner(&mut nodes);
        ParsingResult {
            nodes,
            file_path: self.file_path,
        }
    }

    fn parse_inner(&mut self, nodes: &mut Vec<Node>) {
        let mut nodes = nodes;
        let mut body_str = String::new();
        while self.can_pop() {
            let curr = self.pop();
            match curr {
                Token::MARK { kind } => {
                    nodes.push(Node::DATA {
                        data: body_str.to_string(),
                    });
                    body_str = String::new();
                    self.handle_func(nodes);
                }
                Token::NL => {
                    self.line = self.line + 1;
                    body_str.push('\n');
                }
                tok => {
                    body_str.push_str(&tok.val());
                }
            }
        }
        nodes.push(Node::DATA {
            data: body_str.to_string(),
        });
    }

    fn handle_func(&mut self, nodes: &mut Vec<Node>) {
        let mut nodes = nodes;
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

    fn handle_include(&mut self, nodes: &mut Vec<Node>) {
        let mut path = String::new();

        self.remove_spaces();

        loop {
            match self.peek() {
                Token::IDENT { str } => {
                    path.push_str(&str);
                }
                // include
                Token::DD => {
                    self.pop();
                    nodes.push(Node::INCLUDE {
                        path: path.clone(),
                        line: self.line,
                    });
                    self.remove_till_tl();
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
        let mut conditions: Option<Vec<(String, String, Condition)>> = None;
        let mut body: Option<Box<Node>> = None;

        // get def name
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

        loop {
            match self.peek() {
                // def name:
                Token::DD => {
                    self.pop();
                    self.remove_till_tl();
                    break;
                }
                // def name place name where ...
                Token::PLACE => {
                    self.pop();
                    self.remove_spaces();
                    match self.peek() {
                        Token::IDENT { str: _ } => {
                            self.handle_place(nodes);
                            let place = nodes.remove(nodes.len() - 1);
                            body = Some(Box::new(place));
                            break;
                        }
                        _ => handle_error(
                            format!("Expected ident found {:?} after def place", self.peek()),
                            self.line,
                            self.file_path.clone(),
                        ),
                    }
                }
                // def name when condition
                Token::WHEN => {
                    self.pop();
                    loop {
                        if !self.can_pop() {
                            break;
                        }
                        self.remove_spaces();
                        match self.peek() {
                            // def name were name
                            Token::IDENT { str } => {
                                let var = str;
                                self.pop();
                                self.remove_spaces();
                                match self.peek() {
                                    // def name were name =
                                    Token::EQUALS => {
                                        self.pop();
                                        self.remove_spaces();
                                        match self.peek() {
                                            // def name when name = val
                                            Token::IDENT { str } => {
                                                self.pop();
                                                if conditions.is_none() {
                                                    conditions = Some(Vec::new());
                                                }
                                                conditions.as_mut().unwrap().push((
                                                    var,
                                                    str,
                                                    Condition::EQUALS,
                                                ));
                                                self.remove_spaces();
                                                match self.peek() {
                                                    // def name when name = val:
                                                    Token::DD => {
                                                        self.remove_till_tl();
                                                        break;
                                                    }
                                                    Token::COMMA => {
                                                        self.pop();
                                                        break;
                                                    }
                                                    Token::PLACE => {
                                                        break;
                                                    }
                                                    Token::DEF => {
                                                        break;
                                                    }
                                                    Token::WHERE => {
                                                        break;
                                                    }
                                                    _ => {
                                                        panic!("idk");
                                                    }
                                                }
                                            }
                                            _ => handle_error(
                                                format!(
                                                    "Expected ident found {:?} in def <name> where <name><condition><here>",
                                                    self.peek()
                                                ),
                                                self.line,
                                                self.file_path.clone(),
                                            ),
                                        }
                                    }
                                    _ => handle_error(
                                        format!(
                                            "Expected condition found {:?} in def <name> where <name><here>",
                                            self.peek()
                                        ),
                                        self.line,
                                        self.file_path.clone(),
                                    ),
                                }
                            }
                            Token::PLACE => {
                                break;
                            }
                            _ => handle_error(
                                format!(
                                    "Expected ident found {:?} in def <name> where <here>",
                                    self.peek()
                                ),
                                self.line,
                                self.file_path.clone(),
                            ),
                        }
                    }
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
        }

        // if body is already defined, then its def place
        if body.is_some() {
            nodes.push(Node::DEF {
                name: def_name.to_string(),
                body: body.unwrap(),
                line: self.line,
                conditions: conditions,
            });
            return;
        }
        // body handling
        let body = self.build_body();
        nodes.push(Node::DEF {
            name: def_name.to_string(),
            body: Box::new(body),
            line: self.line,
            conditions: conditions,
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
    fn remove_till_tl(&mut self) {
        loop {
            match self.peek() {
                Token::SPACE => {
                    self.pop();
                }
                Token::NL => {
                    self.pop();
                    return;
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
                // regular var declaration
                Token::VAR => {
                    self.pop();
                    body.push(Node::DATA {
                        data: body_str.to_string(),
                    });
                    body_str = String::new();
                    self.remove_spaces();
                    match self.peek() {
                        // $#ident
                        Token::IDENT { str } => {
                            self.pop();
                            match self.peek() {
                                Token::PLUS => {
                                    self.pop();
                                }
                                _ => (),
                            }
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
                        //- endef:
                        Token::ENDEF => {
                            self.pop();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                            });
                            self.remove_spaces();
                            match self.peek() {
                                // endef :
                                Token::DD => {
                                    self.pop();
                                    self.remove_till_tl();
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
                        /*- $#var -> -*/
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
                                                    match self.peek() {
                                                        Token::IDENT { str } => {
                                                            self.pop();
                                                            body.push(Node::RARROWVAR {
                                                                name,
                                                                default: Some(str.clone()),
                                                            });
                                                            /* $#var -> *///+
                                                            match self.peek() {
                                                                Token::PLUS => {
                                                                    self.pop();
                                                                }
                                                                _ => (),
                                                            }
                                                        }
                                                        tok => {
                                                            self.pop();
                                                            body.push(Node::RARROWVAR {
                                                                name,
                                                                default: Some(tok.val()),
                                                            });
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
                tok => {
                    body_str.push_str(&tok.val());
                }
            }
            self.pop();
        }
        return Node::BODY { data: body };
    }

    fn handle_place(&mut self, nodes: &mut Vec<Node>) {
        // reaches here as //- place
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
            // place ident:
            Token::DD => {
                self.pop();
                nodes.push(Node::PLACE {
                    name: place_id,
                    args: Vec::new(),
                    line: self.line,
                });
                self.remove_till_tl();
                return;
            }
            // place ident were
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
                                            args.push((from, str));
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::COMMA => {
                                                    self.pop();
                                                }
                                                // ident = ident
                                                Token::DD => {
                                                    self.pop();
                                                    nodes.push(Node::PLACE {
                                                        name: place_id.clone(),
                                                        args: args,
                                                        line: self.line,
                                                    });
                                                    self.remove_till_tl();
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
                                                                        self.pop();
                                                                        ends = true;
                                                                        break;
                                                                    }
                                                                    _ => {
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            if ends {
                                                                break;
                                                            } else {
                                                                arg_str.push_str(&spaces);
                                                            }
                                                            // if has " after mark
                                                        }
                                                    }
                                                    tok => {
                                                        arg_str.push_str(&tok.val());
                                                    }
                                                }
                                                self.pop();
                                            }
                                            args.push((from, arg_str));
                                            self.remove_spaces();
                                            // ident = "ident"
                                            match self.peek() {
                                                Token::DD => {
                                                    self.pop();
                                                    nodes.push(Node::PLACE {
                                                        name: place_id.clone(),
                                                        args: args,
                                                        line: self.line,
                                                    });
                                                    self.remove_till_tl();
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
                            self.remove_till_tl();
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
                    "{:?} cant go in //- place <name> <here> at line {}, forgot \":\" or \",\" ?",
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
