use std::str;

use crate::{
    error_handler::{CompilationError, handle_error, handle_error_parser},
    lexer::{Token, TokenResult},
    structs::*,
};

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
        let mut body_str = String::new();
        let mut parser_result = ParsingResult::new(self.file_path.clone());
        while self.can_pop() {
            body_str = self.parse_inner(&mut parser_result, body_str);
        }
        parser_result.push(Node::DATA {
            data: body_str.to_string(),
            line: self.line,
        });
        parser_result
    }

    fn parse_inner(&mut self, nodes: &mut ParsingResult, body_str: String) -> String {
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

    fn handle_func(&mut self, nodes: &mut ParsingResult) {
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

    /// gets the file path from tokens
    /// at this point we know a path is to come but no ident has been consumed
    /// ex: parent/child.txt
    fn handle_path(&mut self) -> String {
        let mut path: String = String::new();
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

    fn handle_derive(&mut self, nodes: &mut ParsingResult) {
        self.remove_spaces();
        let path = match self.peek() {
            Token::IDENT { str:_ } => {
                self.handle_path()
            },
            _ => self.file_path.to_string(), 
        };
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
    fn handle_create(&mut self, nodes: &mut ParsingResult) {
        let path: String = self.handle_path();
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
                self.remove_till_nl();
                return;
            }
            Token::PLACE => {
                self.ptr_next();
                let mut temp_nodes = ParsingResult::new(path.clone());
                // returns one place
                self.handle_place(&mut temp_nodes);
                //let content = Some(Box::new(Node::BODY { data: temp_nodes, line: starting_line }))
                let node = Node::new_create(path, temp_nodes.nodes, starting_line);
                nodes.push(node);
                return;
            }
            _ => handle_error_parser(CompilationError::InvalidAfterFilePath, self),
        }
    }

    fn handle_include(&mut self, nodes: &mut ParsingResult) {
        self.remove_spaces();

        let path = match self.peek() {
            Token::IDENT { str: _ } => self.handle_path(),
            _ => {
                handle_error_parser(CompilationError::InvalidTokenInIncludePath, self);
            }
        };

        self.remove_till_nl();

        match self.pop() {
            Token::DD => {}
            _ => {
                panic!("todo error message")
            }
        }

        nodes.push(Node::INCLUDE {
            path: path.clone(),
            line: self.line,
        });

        return;
    }

    fn handle_def(&mut self, nodes: &mut ParsingResult) {
        //- def ...
        self.remove_spaces();

        let mut conditions: Option<Vec<(String, String, Condition)>> = None;
        let mut defaults: Option<Vec<(String, String)>> = None;
        let mut body: Option<Box<Node>> = None;

        // get def name
        let def_name = match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                str
            }
            Token::PLACE => {
                self.ptr_next();
                "place".to_string()
            }
            _ => {
                handle_error_parser(CompilationError::InvalidDefName, self);
            }
        };

        self.remove_spaces();

        // declaration
        loop {
            match self.peek() {
                // def name:
                Token::DD => {
                    self.ptr_next();
                    self.remove_till_nl();
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
                        _ => handle_error_parser(CompilationError::InvalidDefPlaceName, self),
                    }
                }
                Token::DERIVE => {
                    // todo:
                    self.ptr_next();
                    let mut temp_nodes = ParsingResult::new(self.file_path.clone());
                    self.handle_derive(&mut temp_nodes);
                    body = Some(Box::new(temp_nodes.nodes[0].clone()));
                    break;
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
                                                        self.remove_till_nl();
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
                                                        self.remove_till_nl();
                                                        break;
                                                    }
                                                    Token::COMMA => {
                                                        self.ptr_next();
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

        // if nothing defines a body then its a def of kind /*- def name: ... endef -*/
        // so we need to build the body
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
    fn remove_till_nl(&mut self) {
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

    /// handles variable options 
    /// reaches here at the ident after \
    /// returns a list of the options
    /// ex: $#var\CAMEL
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
            //let mut options_1: Option<Vec<String>> = None;
            let mut options_2: Option<Vec<String>> = None;
            self.remove_spaces();
            match self.peek() {
                Token::IDENT { str } => {
                    self.ptr_next();
                    let from = str;
                    if matches!(self.peek(), Token::BSLASH) {
                        self.ptr_next();
                        //options_1 = self.handle_var_options();
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

    /// handles any mark found inside a body
    /// this is ONLY called coming from a body
    /// already consumed the mark
    fn handle_mark_at_body(&mut self, body_str: &mut String, body: &mut Vec<Node>) -> bool {
        self.remove_spaces();
        match self.peek() {
            //- end:
            Token::END => {
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                self.remove_spaces();
                match self.peek() {
                    // end :
                    Token::DD => {
                        self.ptr_next();
                        self.remove_till_nl();
                        if matches!(self.peek(), Token::NL) {
                            self.line = self.line - 1;
                        }
                        self.unpop();
                        return true;
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
                *body_str = String::new();
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
                                        return false;
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
                    _ => handle_error_parser(CompilationError::InvalidArrowVarName, self),
                }
            }
            Token::DEF => {
                // inner def
                self.ptr_next();
                let mut nodes = ParsingResult::new(self.file_path.clone());
                self.handle_def(&mut nodes);
                body.append(&mut nodes.nodes);
            }
            Token::PLACE => {
                // inner place
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                *body_str = String::new();
                let mut nodes = ParsingResult::new(self.file_path.clone());
                self.handle_place(&mut nodes);
                body.append(&mut nodes.nodes);
                if matches!(self.peek(), Token::NL) {
                    self.line = self.line - 1;
                }
                self.unpop();
            }
            Token::INCLUDE => {
                // inner include
                self.ptr_next();
                let mut nodes = ParsingResult::new(self.file_path.clone());
                self.handle_include(&mut nodes);
                body.append(&mut nodes.nodes);
            }
            Token::MATCH => {
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                *body_str = String::new();
                let node = self.handle_match();
                body.push(node);
            }
            _ => {
                handle_error_parser(CompilationError::InvalidBodyCommand, self);
            }
        }
        return false;
    }

    fn handle_match(&mut self) -> Node {
        self.remove_till_nl();
        let var_name = match self.pop() {
            Token::IDENT { str } => str,
            _ => panic!("todo error message"),
        };
        self.remove_till_nl();
        match self.pop() {
            Token::DD => {}
            _ => panic!("todo error message"),
        };

        let mut matches = Vec::new();
        self.remove_spaces();
        loop {
            self.remove_spaces();
            match self.pop() {
                Token::MARK { kind:_ } => {}
                _ => panic!("forgot mark"),
            }
            self.remove_spaces();
            match self.pop() {
                Token::CASE => {
                    let arm_body = self.handle_match_arm();
                    matches.push(arm_body);
                }
                Token::END => {
                    self.ptr_next();
                    break;
                }
                tok => panic!(
                    "todo error message l: {} expected case found {:?}",
                    self.get_line(),
                    tok
                ),
            }
        }

        return Node::MATCH {
            line: self.line,
            var_name: var_name,
            val: matches,
        };
    }

    /// handles match arm
    /// already poped match token here
    /// returns a body node and the match value inside the match arm struct
    fn handle_match_arm(&mut self) -> MatchArm {
        self.remove_till_nl();
        let match_value = match self.pop() {
            Token::IDENT { str } => str,
            _ => panic!("todo error message"),
        };

        self.remove_till_nl();
        match self.pop() {
            Token::DD => {}
            _ => panic!("todo error message"),
        };
        self.remove_till_nl();

        let body = self.build_body();
        MatchArm::new(match_value, body)
    }

    /// builds a body Node
    /// contains raw text and any nodes supported inside of a def body
    /// ends at "end"
    /// comes from def or match arm
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
                            let mut option = None; 
                            match self.peek() {
                                Token::PLUS => {
                                    self.ptr_next();
                                },
                                Token::BSLASH => {
                                    self.ptr_next();
                                    option = self.handle_var_options();
                                    match self.peek() {
                                        Token::PLUS => {
                                            self.ptr_next();
                                        }
                                        _ => (),
                                    }
                                },
                                _ => (),
                            }
                            body.push(Node::var_template(str,option));
                            continue;
                        }
                        // $#name
                        _ => {
                            handle_error_parser(CompilationError::InvalidVar, self);
                        }
                    }
                }
                Token::MARK { kind: _ } => {
                    self.ptr_next();
                    let should_break = self.handle_mark_at_body(&mut body_str, &mut body);
                    if should_break {
                        break;
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

    fn handle_place(&mut self, nodes: &mut ParsingResult) {
        // reaches here as //- place
        self.remove_spaces();

        let place_id = match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                str
            }
            Token::PLACE => {
                self.ptr_next();
                "place".to_string()
            }
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
                    self.remove_till_nl();
                    break;
                }
                // place ident were
                Token::WHERE => {
                    self.ptr_next();
                    args.append(&mut self.handle_var());
                    self.remove_till_nl();
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
}
