use core::panic;

use crate::{
    error_handler::{handle_error, handle_expected_error},
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
        defaults: Option<Vec<(String,String)>>,
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
    CREATE {
        path: String,
        content: Option<Box<Node>>,
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
    fn peek_behind(&mut self, i: usize) -> Token {
        self.tokens[self.ptr - i].clone()
    }
    fn peek_ahead(&mut self, i: usize) -> Token {
        self.tokens[self.ptr + i].clone()
    }
    fn ptr_next(&mut self) {
        self.ptr = self.ptr + 1;
    }
    fn unpop(&mut self) {
        self.ptr = self.ptr - 1;
    }
    fn can_pop(&self) -> bool {
        self.tokens.len() > self.ptr
    }

    pub fn parse(mut self) -> ParsingResult {
        let mut nodes: Vec<Node> = Vec::new();
        let mut body_str = String::new();
        while self.can_pop() {
            body_str = self.parse_inner(&mut nodes, body_str);
        }
        nodes.push(Node::DATA {
            data: body_str.to_string(),
        });
        ParsingResult {
            nodes,
            file_path: self.file_path,
        }
    }

    fn parse_inner(&mut self, nodes: &mut Vec<Node>, body_str: String) -> String {
        let curr = self.pop();
        let mut body_str = body_str;
        match curr {
            Token::MARK { kind: _ } => {
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
        return body_str;
    }

    fn handle_func(&mut self, nodes: &mut Vec<Node>) {
        let mut nodes = nodes;
        self.remove_spaces();
        match self.peek() {
            Token::DEF => {
                self.ptr_next();
                self.handle_def(&mut nodes);
            }
            Token::PLACE => {
                self.ptr_next();
                self.handle_place(&mut nodes);
            }
            Token::INCLUDE => {
                self.ptr_next();
                self.handle_include(&mut nodes);
            }
            Token::CREATE => {
                self.ptr_next();
                self.handle_create(&mut nodes);
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

    // create filepath place defname:
    fn handle_create(&mut self, nodes: &mut Vec<Node>) {

        let mut path: String = String::new();
        // filepath
        // ex: parent/child.txt
        self.remove_spaces();
        loop {
            match self.peek() {
                Token::IDENT { str } => {
                    self.ptr_next();
                    path.push_str(&str);
                },
                Token::SPACE => {
                    self.ptr_next();
                    break;
                },
                _ => {
                    handle_error(format!("Expected file name found {:?}", self.peek()), self.line, self.file_path.clone())
                }
            }
        }
        self.remove_spaces();

        match self.peek() {
            Token::DD => {
                self.ptr_next();
                nodes.push(Node::CREATE { path, content: None });
                return;
            },
            Token::PLACE => {
                self.ptr_next();
                let mut temp_nodes: Vec<Node> = Vec::new();
                // returns one place
                self.handle_place(&mut temp_nodes);
                nodes.push(Node::CREATE { path, content: Some(Box::new(Node::BODY { data: temp_nodes })) });
                return;
            },
            _ => {
                handle_error(format!("Found {:?} wich is invalid in create",self.peek()), self.line, self.file_path.clone());
            },
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
                    println!("DD1");
                    self.ptr_next();
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
            self.ptr_next();
        }
    }

    fn handle_def(&mut self, nodes: &mut Vec<Node>) {
        //- def ...

        self.remove_spaces();

        let mut def_name = String::new();
        let mut conditions: Option<Vec<(String, String, Condition)>> = None;
        let mut defaults: Option<Vec<(String,String)>> = None;
        let mut body: Option<Box<Node>> = None;

        // get def name
        match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
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

        // declaration
        loop {
            match self.peek() {
                // def name:
                Token::DD => {
                    println!("DD2");
                    self.ptr_next();
                    self.remove_till_tl();
                    break;
                }
                // def name place name where ...
                Token::PLACE => {
                    self.ptr_next();
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
                    self.ptr_next();
                    loop {
                        if !self.can_pop() {
                            break;
                        }
                        self.remove_spaces();
                        match self.peek() {
                            // def name were name
                            Token::IDENT { str } => {
                                let var = str;
                                self.ptr_next();
                                self.remove_spaces();
                                match self.peek() {
                                    // def name were name =
                                    Token::EQUALS => {
                                        self.ptr_next();
                                        self.remove_spaces();
                                        match self.peek() {
                                            // def name when name = val
                                            Token::IDENT { str } => {
                                                self.ptr_next();
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
                                                        println!("DD3");
                                                        self.remove_till_tl();
                                                        break;
                                                    }
                                                    Token::COMMA => {
                                                        self.ptr_next();
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
                },
                // def where
                Token::WHERE => {
                    self.ptr_next();
                    loop {
                        self.remove_spaces();
                        match self.peek() {
                            Token::IDENT { str } => {
                                let var = str;
                                self.ptr_next();
                                self.remove_spaces();
                                match self.peek() {
                                    Token::EQUALS => {
                                        self.ptr_next();
                                        self.remove_spaces();
                                        match self.peek() {
                                            Token::IDENT { str } => {
                                                self.ptr_next();
                                                self.remove_spaces();
                                                let val = str;
                                                if defaults.is_none() {
                                                    defaults = Some(Vec::new());
                                                }
                                                defaults.as_mut().unwrap().push((var,val));
                                                match self.peek() {
                                                    Token::DD => {
                                                        self.remove_till_tl();
                                                        break;
                                                    },
                                                    Token::COMMA => {
                                                        continue;
                                                    },
                                                    Token::NL => {
                                                        handle_error("Expected : found newline", self.line, &self.file_path);
                                                    }
                                                    _ => {
                                                        break;
                                                    }
                                                }
                                            },
                                            _ => handle_error(format!("invalid token in def defaults {:?}", self.peek()), self.line, self.file_path.clone())
                                        }
                                    }
                                    _ => handle_error(format!("invalid token in def defaults {:?}", self.peek()), self.line, self.file_path.clone())
                                }
                            },
                            _ => handle_error(format!("invalid token in def defaults {:?}", self.peek()), self.line, self.file_path.clone())
                        }
                    }
                },
                _ => {
                    panic!(
                        "{:?} is invalid in def declaration of name {} after {:?} in line {}",
                        self.peek(),
                        def_name,
                        self.peek_behind(1),
                        self.line
                    );
                }
            }
        }

        // if body is already defined, then its def place
        if body.is_some() {
            println!("Pushed already defined def body to nodes");
            nodes.push(Node::DEF {
                name: def_name.to_string(),
                body: body.unwrap(),
                line: self.line,
                conditions: conditions,
                defaults: defaults,
            });
            return;
        }

        // body handling
        let body = self.build_body();
        println!("pushed body {:?} in def", body);
        nodes.push(Node::DEF {
            name: def_name.to_string(),
            body: Box::new(body),
            line: self.line,
            conditions: conditions,
            defaults: defaults.clone(),
        });
        println!("Body nodes {:?}", nodes);
    }

    fn remove_spaces(&mut self) {
        loop {
            match self.peek() {
                Token::SPACE => {
                    self.ptr_next();
                }
                _ => {
                    return;
                }
            }
        }
    }
    fn remove_till_tl(&mut self) {
        loop {
            match self.peek() {
                Token::SPACE => {
                    self.ptr_next();
                }
                Token::NL => {
                    self.ptr_next();
                    self.line = self.line + 1;
                    return;
                }
                _ => {
                    return;
                }
            }
        }
    }

    // ends at endef
    fn build_body(&mut self) -> Node {
        println!("build body");
        let mut body_str = String::new();
        let mut body: Vec<Node> = Vec::new();
        loop {
            println!("parser peek {:?}", self.peek());
            match self.peek() {
                // regular var declaration
                Token::VAR => {
                    self.ptr_next();
                    body.push(Node::DATA {
                        data: body_str.to_string(),
                    });
                    body_str = String::new();
                    self.remove_spaces();
                    match self.peek() {
                        // $#ident
                        Token::IDENT { str } => {
                            self.ptr_next();
                            match self.peek() {
                                Token::PLUS => {
                                    self.ptr_next();
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
                    self.ptr_next();
                    self.remove_spaces();
                    match self.peek() {
                        //- endef:
                        Token::ENDEF => {
                            println!("endef");
                            self.ptr_next();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                            });
                            self.remove_spaces();
                            match self.peek() {
                                // endef :
                                Token::DD => {
                                    println!("DD4");
                                    self.ptr_next();
                                    self.remove_till_tl();
                                    self.unpop();
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
                            self.ptr_next();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                            });
                            body_str = String::new();
                            match self.peek() {
                                /*- #$ident -> -*/
                                Token::IDENT { str } => {
                                    let name = str;
                                    self.ptr_next();
                                    let spaces = self.collect_spaces();
                                    match self.peek() {
                                        Token::RARROW => {
                                            self.ptr_next();
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::MARK { kind } => {
                                                    self.ptr_next();
                                                    self.remove_spaces();
                                                    match self.peek() {
                                                        Token::IDENT { str } => {
                                                            self.ptr_next();
                                                            body.push(Node::RARROWVAR {
                                                                name,
                                                                default: Some(str.clone()),
                                                            });
                                                            /* $#var -> *///+
                                                            match self.peek() {
                                                                Token::PLUS => {
                                                                    self.ptr_next();
                                                                }
                                                                _ => (),
                                                            }
                                                        }
                                                        Token::NL => {
                                                            handle_error(
                                                                "Newline not cannot proced an arrow variable",
                                                                self.line,
                                                                &self.file_path,
                                                            );
                                                        }
                                                        tok => {
                                                            self.ptr_next();
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
                        Token::DEF => {
                            // inner def
                            self.ptr_next();
                            let mut nodes: Vec<Node> = Vec::new();
                            println!("inner def");
                            self.handle_def(&mut nodes);
                            println!("inner def end");
                            body.append(&mut nodes);
                            println!("Appeded to body {:?}", nodes);
                        }
                        Token::PLACE => {
                            // inner place
                            self.ptr_next();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                            });
                            body_str = String::new();
                            let mut nodes: Vec<Node> = Vec::new();
                            self.handle_place(&mut nodes);
                            body.append(&mut nodes);
                            self.unpop();
                        }
                        Token::INCLUDE => {
                            self.ptr_next();
                            let mut nodes: Vec<Node> = Vec::new();
                            self.handle_include(&mut nodes);
                            body.append(&mut nodes);
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
                Token::EOF => handle_error("Found EOF inside a body", self.line, &self.file_path),
                Token::NL => {
                    body_str.push_str("\n");
                    self.line = self.line + 1;
                }
                tok => {
                    body_str.push_str(&tok.val());
                }
            }
            self.ptr_next();
        }
        println!("returned {:?}", body);
        return Node::BODY { data: body };
    }

    fn handle_place(&mut self, nodes: &mut Vec<Node>) {
        // reaches here as //- place
        self.remove_spaces();

        let place_id = match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                str
            }
            _ => {
                panic!(
                    "{:?} cant go after PLACE in line {}",
                    self.peek(),
                    self.line
                )
            }
        };

        self.remove_spaces();
        match self.peek() {
            // place ident:
            Token::DD => {
                println!("DD5");
                self.ptr_next();
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
                    self.ptr_next();
                    self.remove_spaces();
                    match self.peek() {
                        Token::IDENT { str } => {
                            self.ptr_next();
                            self.remove_spaces();
                            let from = str;
                            match self.peek() {
                                Token::EQUALS => {
                                    self.ptr_next();
                                    self.remove_spaces();
                                    match self.peek() {
                                        // ident = ident -> variable assignement
                                        Token::IDENT { str } => {
                                            self.ptr_next();
                                            args.push((from, str));
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::COMMA => {
                                                    self.ptr_next();
                                                }
                                                // ident = ident
                                                Token::DD => {
                                                    println!("DD6");
                                                    self.ptr_next();
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
                                            self.ptr_next();
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
                                                            self.ptr_next();
                                                            break;
                                                        }
                                                    }
                                                    Token::MARK { kind } => {
                                                        self.ptr_next();
                                                        if !has_new_line {
                                                            arg_str.push_str(&kind);
                                                        } else {
                                                            // if value has a newline after the first ", then ends at mark + "
                                                            let mut spaces = String::new();
                                                            let mut ends = false;
                                                            loop {
                                                                match self.peek() {
                                                                    Token::SPACE => {
                                                                        self.ptr_next();
                                                                        spaces.push(' ');
                                                                    }
                                                                    Token::DQUOTE => {
                                                                        self.ptr_next();
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
                                                self.ptr_next();
                                            }
                                            args.push((from, arg_str));
                                            self.remove_spaces();
                                            // ident = "ident"
                                            match self.peek() {
                                                Token::DD => {
                                                    println!("DD7");
                                                    self.ptr_next();
                                                    nodes.push(Node::PLACE {
                                                        name: place_id.clone(),
                                                        args: args,
                                                        line: self.line,
                                                    });
                                                    self.remove_till_tl();
                                                    return;
                                                }
                                                Token::COMMA => {
                                                    self.ptr_next();
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
                            println!("DD8");
                            self.ptr_next();
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
                    self.ptr_next();
                    spaces.push(' ');
                }
                Token::DQUOTE => {
                    self.ptr_next();
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
