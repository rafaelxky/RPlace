use core::panic;
use std::{str, vec};

use crate::{
    error_handler::{CompilationError, handle_error, handle_error_parser, handle_expected_error},
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
pub enum ValueType {
    Literal,
    Var,
}
#[derive(Debug, Clone)]
pub struct Value {
    pub value_type: ValueType,
    pub value: String,
    pub options: Option<Vec<String>>,
}
impl ToString for Value {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    // - def template_1
    DEF {
        conditions: Option<Vec<(String, String, Condition)>>,
        defaults: Option<Vec<(String, String)>>,
        name: String,
        body: Box<Node>,
        line: usize,
    },
    // either data or var ($a)
    BODY {
        data: Vec<Node>,
        line: usize,
    },
    // def body
    DATA {
        data: String,
        line: usize,
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
        args: Vec<(String, Value)>,
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
    DERIVE {
        path: String,
        val: Vec<(String, Value)>,
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
    pub fn get_line(&self) -> usize {
        self.line
    }
    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
    pub fn peek(&self) -> Token {
        self.tokens[self.ptr].clone()
    }
    fn pop(&mut self) -> Token {
        self.ptr = self.ptr + 1;
        self.tokens[self.ptr - 1].clone()
    }
    fn peek_behind(&self, i: usize) -> Token {
        self.tokens[self.ptr - i].clone()
    }
    fn peek_ahead(&self, i: usize) -> Token {
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
    pub fn get_tok_around(&self, dist: usize) -> String {
        let mut str = String::new();
        for i in (1..=dist).rev() {
            str.push_str(&self.peek_behind(i).val());
        }
        str.push_str(&self.peek().val());
        for i in 1..=dist {
            str.push_str(&self.peek_ahead(i).val());
        }
        return str;
    }
    pub fn get_tok_around_colored(&self, dist: usize) -> String {
        let red = "\x1b[31m";
        let reset = "\x1b[0m";
        let mut str = String::new();
        let behind = self.ptr.min(dist);
        let ahead = (self.tokens.len() - 1 - self.ptr).min(dist);
        for i in (1..=behind).rev() {
            str.push_str(&self.peek_behind(i).val());
        }
        str.push_str(&format!("{}{}{}", red, &self.peek().val(), reset));
        for i in 1..=ahead {
            str.push_str(&self.peek_ahead(i).val());
        }
        return str;
    }

    pub fn parse(mut self) -> ParsingResult {
        let mut nodes: Vec<Node> = Vec::new();
        let mut body_str = String::new();
        while self.can_pop() {
            body_str = self.parse_inner(&mut nodes, body_str);
        }
        nodes.push(Node::DATA {
            data: body_str.to_string(),
            line: self.line,
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
                    line: self.line,
                });
                body_str = String::new();
                self.handle_func(nodes);
            }
            Token::NL => {
                self.line = self.line + 1;
                //println!("parse inner newline {} at {}", self.line, self.get_tok_around_colored(10));
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
            Token::DERIVE => {
                self.ptr_next();
                self.handle_derive(&mut nodes);
            }
            _ => {
                handle_error_parser(CompilationError::InvalidFunc, self);
            }
        }
    }

    fn get_path(&mut self) -> String {
        let mut path: String = String::new();
        // filepath
        // ex: parent/child.txt
        self.remove_spaces();
        loop {
            match self.peek() {
                Token::IDENT { str } => {
                    self.ptr_next();
                    path.push_str(&str);
                }
                Token::SPACE => {
                    self.ptr_next();
                    break;
                }
                Token::DD => {
                    break;
                }
                _ => handle_error_parser(CompilationError::InvalidTokenInPath, self),
            }
        }
        path
    }

    fn handle_derive(&mut self, nodes: &mut Vec<Node>) {
        let path: String = self.get_path();
        self.remove_spaces();

        // derive options
        let args = match self.peek() {
            Token::WHERE => {
                self.ptr_next();
                self.remove_spaces();
                let args = self.handle_var();
                match self.peek() {
                    Token::DD => {
                        self.ptr_next();
                        args
                    }
                    _ => handle_error_parser(CompilationError::InvalidDeriveOption, self),
                }
            }
            _ => handle_error_parser(CompilationError::InvalidDeriveOption, self),
        };
        nodes.push(Node::DERIVE {
            path: path,
            val: args,
        });
    }

    // create filepath place defname:
    fn handle_create(&mut self, nodes: &mut Vec<Node>) {
        let path: String = self.get_path();
        let starting_line = self.get_line();
        // filepath
        // ex: parent/child.txt

        self.remove_spaces();

        match self.peek() {
            Token::DD => {
                self.ptr_next();
                nodes.push(Node::CREATE {
                    path,
                    content: None,
                });
                self.remove_till_tl();
                return;
            }
            Token::PLACE => {
                self.ptr_next();
                let mut temp_nodes: Vec<Node> = Vec::new();
                // returns one place
                self.handle_place(&mut temp_nodes);
                nodes.push(Node::CREATE {
                    path,
                    content: Some(Box::new(Node::BODY {
                        data: temp_nodes,
                        line: starting_line,
                    })),
                });
                return;
            }
            _ => handle_error_parser(CompilationError::InvalidAfterFilePath, self),
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
                    handle_error_parser(CompilationError::InvalidTokenInIncludePath, self);
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
        let mut defaults: Option<Vec<(String, String)>> = None;
        let mut body: Option<Box<Node>> = None;

        // get def name
        match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                def_name = str;
            }
            Token::PLACE => {
                self.ptr_next();
                def_name = "place".to_string();
            }
            _ => {
                handle_error_parser(CompilationError::InvalidDefName, self);
            }
        }

        self.remove_spaces();

        // declaration
        loop {
            match self.peek() {
                // def name:
                Token::DD => {
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
                        },
                        _ => handle_error_parser(CompilationError::InvalidDefPlaceName, self),
                    }
                },
                Token::DERIVE => {
                    // todo:
                    self.ptr_next();
                    let mut temp_nodes: Vec<Node> = Vec::new();
                    self.handle_derive(&mut temp_nodes);
                    body = Some(Box::new(temp_nodes[0].clone()));
                    break;
                },
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
                                                    // def name when name = val <here>
                                                    _ => {
                                                        handle_error_parser(
                                                            CompilationError::InvalidFinishTokWhen,
                                                            self,
                                                        );
                                                    }
                                                }
                                            }
                                            // def name when name = <here>
                                            _ => handle_error_parser(
                                                CompilationError::Invalid2ndIdentWhen,
                                                self,
                                            ),
                                        }
                                    }
                                    _ => handle_error_parser(
                                        CompilationError::InvalidComparissonTok,
                                        self,
                                    ),
                                }
                            }
                            Token::PLACE => {
                                break;
                            }
                            _ => {
                                handle_error_parser(CompilationError::Invalid1stIdentWhen, self);
                            }
                        }
                    }
                }
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
                                                defaults.as_mut().unwrap().push((var, val));
                                                match self.peek() {
                                                    Token::DD => {
                                                        self.remove_till_tl();
                                                        break;
                                                    }
                                                    Token::COMMA => {
                                                        continue;
                                                    }
                                                    _ => {
                                                        break;
                                                    }
                                                }
                                            }
                                            // def a where a = <here>
                                            _ => {
                                                handle_error_parser(
                                                    CompilationError::Invalid2ndIdentDefWhere,
                                                    self,
                                                );
                                            }
                                        }
                                    }
                                    _ => {
                                        handle_error_parser(
                                            CompilationError::InvalidAssignementDefWhere,
                                            self,
                                        );
                                    }
                                }
                            }
                            _ => {
                                handle_error_parser(
                                    CompilationError::Invalid1stIdentDefWhere,
                                    self,
                                );
                            }
                        }
                    }
                }
                _ => {
                    handle_error_parser(CompilationError::InvalidDefOption, self);
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
                defaults: defaults,
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
            defaults: defaults.clone(),
        });
    }

    fn remove_spaces(&mut self) {
        loop {
            match self.peek() {
                Token::SPACE => {
                    self.ptr_next();
                }
                Token::NL => {
                    self.line = self.line + 1;
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

    fn handle_var_options(&mut self) -> Option<Vec<String>> {
        let mut options: Option<Vec<String>> = None;
        loop {
            match self.peek() {
                Token::IDENT { str } => {
                    self.ptr_next();
                    if options.is_none() {
                        options = Some(Vec::new());
                    }
                    options.as_mut().unwrap().push(str);
                    match self.peek() {
                        Token::BSLASH => {
                            self.ptr_next();
                            continue;
                        }
                        _ => break,
                    }
                }
                _ => match self.peek().try_get_soft_keyword() {
                    Some(str) => {
                        self.ptr_next();
                        if options.is_none() {
                            options = Some(Vec::new());
                        }
                        options.as_mut().unwrap().push(str);
                        match self.peek() {
                            Token::BSLASH => {
                                self.ptr_next();
                                continue;
                            }
                            _ => break,
                        }
                    }
                    None => handle_error_parser(CompilationError::InvalidVarOption, self),
                },
            }
        }
        options
    }

    fn handle_var(&mut self) -> Vec<(String, Value)> {
        let mut args: Vec<(String, Value)> = Vec::new();
        loop {
            let mut options_1: Option<Vec<String>> = None;
            let mut options_2: Option<Vec<String>> = None;
            self.remove_spaces();
            match self.peek() {
                Token::IDENT { str } => {
                    self.ptr_next();
                    let from = str;
                    if matches!(self.peek(), Token::BSLASH) {
                        self.ptr_next();
                        options_1 = self.handle_var_options();
                    }
                    self.remove_spaces();
                    match self.peek() {
                        Token::EQUALS => {
                            self.ptr_next();
                            self.remove_spaces();
                            match self.peek() {
                                // ident = ident -> variable assignement
                                Token::IDENT { str } => {
                                    self.ptr_next();
                                    if matches!(self.peek(), Token::BSLASH) {
                                        self.ptr_next();
                                        options_2 = self.handle_var_options();
                                    }
                                    self.remove_spaces();
                                    args.push((
                                        from,
                                        Value {
                                            value_type: ValueType::Literal,
                                            value: str,
                                            options: options_2,
                                        },
                                    ));
                                    match self.peek() {
                                        Token::COMMA => {
                                            self.ptr_next();
                                            continue;
                                        }
                                        // ident = ident
                                        Token::DD => {
                                            return args;
                                        }
                                        // second ident
                                        _ => {
                                            return args;
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
                                            handle_error_parser(
                                                CompilationError::EOFInQuotVar,
                                                self,
                                            );
                                        }
                                        match self.peek() {
                                            Token::NL => {
                                                arg_str.push('\n');
                                                self.line = self.line + 1;
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
                                    if matches!(self.peek(), Token::BSLASH) {
                                        self.ptr_next();
                                        options_2 = self.handle_var_options();
                                    }
                                    self.remove_spaces();
                                    args.push((
                                        from,
                                        Value {
                                            value_type: ValueType::Literal,
                                            value: arg_str,
                                            options: options_2,
                                        },
                                    ));
                                    // ident = "ident"
                                    match self.peek() {
                                        Token::DD => {
                                            return args;
                                        }
                                        Token::COMMA => {
                                            self.ptr_next();
                                        }
                                        _ => {
                                            return args;
                                        }
                                    }
                                }
                                Token::VAR => {
                                    self.ptr_next();
                                    match self.peek() {
                                        Token::IDENT { str } => {
                                            self.ptr_next();
                                            args.push((
                                                from,
                                                Value {
                                                    value_type: ValueType::Var,
                                                    value: str,
                                                    options: None,
                                                },
                                            ));
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::DD => {
                                                    return args;
                                                }
                                                Token::COMMA => {
                                                    self.ptr_next();
                                                }
                                                _ => {
                                                    return args;
                                                }
                                            }
                                        }
                                        _ => handle_error(
                                            format!(
                                                "Expected Ident found {:?} at place with variable value",
                                                self.peek()
                                            ),
                                            self.line,
                                            self.file_path.clone(),
                                        ),
                                    }
                                }
                                _ => {
                                    handle_error_parser(CompilationError::Invalid2ndPlaceVar, self);
                                }
                            }
                        }
                        _ => {
                            handle_error_parser(CompilationError::InvalidPlaceAssign, self);
                        }
                    }
                }
                Token::DD => {
                    return args;
                }
                _ => handle_error_parser(CompilationError::Invalid1stPlaceVar, self),
            }
        }
    }

    // ends at endef
    fn build_body(&mut self) -> Node {
        let mut body_str = String::new();
        let mut body: Vec<Node> = Vec::new();
        let line_start = self.line;
        loop {
            match self.peek() {
                // regular var declaration
                Token::VAR => {
                    self.ptr_next();
                    body.push(Node::DATA {
                        data: body_str.to_string(),
                        line: self.line,
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
                        // $#name
                        _ => {
                            handle_error_parser(CompilationError::InvalidVar, self);
                        }
                    }
                }
                Token::MARK { kind } => {
                    self.ptr_next();
                    self.remove_spaces();
                    match self.peek() {
                        //- endef:
                        Token::ENDEF => {
                            self.ptr_next();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                                line: self.line,
                            });
                            self.remove_spaces();
                            match self.peek() {
                                // endef :
                                Token::DD => {
                                    self.ptr_next();
                                    self.remove_till_tl();
                                    if matches!(self.peek(), Token::NL) {
                                        self.line = self.line - 1;
                                    }
                                    self.unpop();
                                    break;
                                }
                                _ => {
                                    handle_error_parser(CompilationError::NoDDEndef, self);
                                }
                            }
                        }
                        /*- $#var -> -*/
                        Token::VAR => {
                            self.ptr_next();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                                line: self.line,
                            });
                            body_str = String::new();
                            match self.peek() {
                                /*- #$ident -> -*/
                                Token::IDENT { str } => {
                                    let name = str;
                                    self.ptr_next();
                                    self.remove_spaces();
                                    match self.peek() {
                                        Token::RARROW => {
                                            self.ptr_next();
                                            self.remove_spaces();
                                            match self.peek() {
                                                Token::MARK { kind: _ } => {
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
                                                            handle_error_parser(
                                                                CompilationError::NLArrowVarName,
                                                                self,
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
                                                _ => handle_error_parser(
                                                    CompilationError::NotMarkAfterArrowVar,
                                                    self,
                                                ),
                                            }
                                        }
                                        _ => handle_error_parser(CompilationError::NotArrow, self),
                                    }
                                }
                                _ => {
                                    handle_error_parser(CompilationError::InvalidArrowVarName, self)
                                }
                            }
                        }
                        Token::DEF => {
                            // inner def
                            self.ptr_next();
                            let mut nodes: Vec<Node> = Vec::new();
                            self.handle_def(&mut nodes);
                            body.append(&mut nodes);
                        }
                        Token::PLACE => {
                            // inner place
                            self.ptr_next();
                            body.push(Node::DATA {
                                data: body_str.to_string(),
                                line: self.line,
                            });
                            body_str = String::new();
                            let mut nodes: Vec<Node> = Vec::new();
                            self.handle_place(&mut nodes);
                            body.append(&mut nodes);
                            if matches!(self.peek(), Token::NL) {
                                self.line = self.line - 1;
                            }
                            self.unpop();
                        }
                        Token::INCLUDE => {
                            self.ptr_next();
                            let mut nodes: Vec<Node> = Vec::new();
                            self.handle_include(&mut nodes);
                            body.append(&mut nodes);
                        }
                        _ => {
                            handle_error_parser(CompilationError::InvalidBodyCommand, self);
                        }
                    }
                }
                Token::EOF => handle_error_parser(CompilationError::BodyEOF, self),
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
        return Node::BODY {
            data: body,
            line: line_start,
        };
    }

    fn handle_place(&mut self, nodes: &mut Vec<Node>) {
        // reaches here as //- place
        self.remove_spaces();

        let place_id = match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                str
            },
            Token::PLACE => {
                self.ptr_next();
                "place".to_string()
            },
            _ => {
                handle_error_parser(CompilationError::InvalidPlaceName, self);
            }
        };

        self.remove_spaces();
        let mut args = Vec::new();
        let place_line = self.line;
        loop {
            match self.peek() {
                // place ident:
                Token::DD => {
                    self.ptr_next();
                    self.remove_till_tl();
                    break;
                }
                // place ident were
                Token::WHERE => {
                    self.ptr_next();
                    args.append(&mut self.handle_var());
                    self.remove_till_tl();
                }
                _ => handle_error_parser(CompilationError::InvalidPlaceOption, self),
            }
        }
        nodes.push(Node::PLACE {
            name: place_id,
            args: args,
            line: place_line,
        });
        return;
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
