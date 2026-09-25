use std::str;
use anyhow::{Ok, Result};

use crate::{
    error_handler::{parser_error, CompilationError},
    lexer::{
        Token::{self},
        TokenResult,
    },
    structs::*,
};

pub mod parse_instructions;
pub mod parse_variables;

pub struct Parser {
    tokens: Vec<Token>,
    ptr: usize,
    line: usize,
    file_path: String,
    project_src: String,
    output_src: String,
}
impl Parser {
    pub fn new(tokens: TokenResult, project_src: String, output_src: String) -> Self {
        Self {
            tokens: tokens.tokens,
            ptr: 0,
            line: 0,
            file_path: tokens.file_path,
            project_src,
            output_src,
        }
    }
    pub fn get_line(&self) -> usize {
        self.line
    }
    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
    pub fn peek(&self) -> Token {
        self.tokens.get(self.ptr).cloned().unwrap_or(Token::EOF)
    }
    fn pop(&mut self) -> Token {
        self.ptr = self.ptr + 1;
        self.tokens[self.ptr - 1].clone()
    }
    fn unpop(&mut self) {
        self.ptr = self.ptr - 1;
    }
    fn peek_behind(&self, i: usize) -> Token {
        self.tokens[self.ptr - i].clone()
    }
    fn peek_ahead(&self, i: usize) -> Token {
        self.tokens
            .get(self.ptr.saturating_add(i))
            .cloned()
            .unwrap_or(Token::EOF)
    }
    fn ptr_next(&mut self) {
        self.ptr = self.ptr + 1;
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
        let ahead = self.tokens.len().saturating_sub(self.ptr + 1).min(dist);
        for i in (1..=behind).rev() {
            str.push_str(&self.peek_behind(i).val());
        }
        str.push_str(&format!("{}{}{}", red, &self.peek().val(), reset));
        for i in 1..=ahead {
            str.push_str(&self.peek_ahead(i).val());
        }
        return str;
    }

    pub fn parse(mut self) -> Result<ParsingResult> {
        let mut body_str = String::new();
        let mut parser_result = ParsingResult::new(self.file_path.clone());
        while self.can_pop() {
            body_str = self.parse_inner(&mut parser_result, body_str)?;
        }
        parser_result.push(Node::DATA {
            data: body_str.to_string(),
            line: self.line,
        });
        Ok(parser_result)
    }

    fn parse_inner(&mut self, nodes: &mut ParsingResult, body_str: String) -> Result<String> {
        let curr = self.pop();
        let mut body_str = body_str;
        match curr {
            Token::MARK { kind: _ } => {
                nodes.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                body_str = String::new();
                self.handle_func(nodes)?;
            }
            Token::NL => {
                self.line = self.line + 1;
                body_str.push('\n');
            }
            tok => {
                body_str.push_str(&tok.val());
            }
        }
        return Ok(body_str);
    }

    fn handle_func(&mut self, nodes: &mut ParsingResult) -> Result<()>{
        let mut nodes = nodes;
        self.remove_spaces();
        match self.peek() {
            Token::DEF => {
                self.ptr_next();
                self.handle_def(&mut nodes)?;
            }
            Token::PLACE => {
                self.ptr_next();
                self.handle_place(&mut nodes)?;
            }
            Token::INCLUDE => {
                self.ptr_next();
                self.handle_include(&mut nodes)?;
            }
            Token::CREATE => {
                self.ptr_next();
                let node = self.handle_create()?;
                nodes.push(node);
            }
            Token::DERIVE => {
                self.ptr_next();
                self.handle_derive(&mut nodes)?;
            }
            Token::VAR => {
                self.ptr_next();
                self.handle_set_variable(&mut nodes)?;
            }
            Token::PARSE => {
                self.ptr_next();
                self.handle_parse_instr(&mut nodes)?;
            }
            Token::MOD => {
                self.ptr_next();
                self.handle_mod(&mut nodes)?;
            }
            _ => {
                return Err(parser_error(CompilationError::InvalidFunc, self));
            }
        }
        Ok(())
    }

    fn handle_def(&mut self, nodes: &mut ParsingResult) -> Result<()>{
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
                return Err(parser_error(CompilationError::InvalidDefName, self));
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
                            self.handle_place(nodes)?;
                            let place = nodes.remove(nodes.len() - 1);
                            body = Some(Box::new(place));
                            break;
                        }
                        _ => {
                            return Err(parser_error(CompilationError::InvalidDefPlaceName, self));
                        },
                    }
                }
                Token::DERIVE => {
                    // todo:
                    self.ptr_next();
                    let mut temp_nodes = ParsingResult::new(self.file_path.clone());
                    self.handle_derive(&mut temp_nodes)?;
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
                                                    Token::COMMA => {
                                                        self.ptr_next();
                                                        continue;
                                                    }
                                                    _ => break,
                                                }
                                            }
                                            // def name when name = <here>
                                            _ => {
                                                return Err(parser_error(
                                                    CompilationError::Invalid2ndIdentWhen,
                                                    self,
                                                ));
                                        },
                                        }
                                    }
                                    _ => {
                                        return Err(parser_error(
                                            CompilationError::InvalidComparissonTok,
                                            self,
                                        ));
                                },
                                }
                            }
                            Token::PLACE => {
                                break;
                            }
                            _ => {
                                return Err(parser_error(CompilationError::Invalid1stIdentWhen, self));
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
                                                        self.ptr_next();
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
                                                return Err(parser_error(
                                                    CompilationError::Invalid2ndIdentDefWhere,
                                                    self,
                                                ));
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(parser_error(
                                            CompilationError::InvalidAssignementDefWhere,
                                            self,
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(parser_error(
                                    CompilationError::Invalid1stIdentDefWhere,
                                    self,
                                ));
                            }
                        }
                    }
                }
                _ => {
                    return Err(parser_error(CompilationError::InvalidDefOption, self));
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
            return Ok(());
        }

        // if nothing defines a body then its a def of kind /*- def name: ... endef -*/
        // so we need to build the body
        let body = self.build_body()?;
        nodes.push(Node::DEF {
            name: def_name.to_string(),
            body: Box::new(body),
            line: self.line,
            conditions: conditions,
            defaults: defaults.clone(),
        });
        return Ok(());
    }
    fn remove_and_return_spaces(&mut self) -> String {
        let mut spaces = String::new();
        loop {
            match self.peek() {
                Token::SPACE => {
                    spaces.push(' ');
                    self.ptr_next();
                }
                Token::NL => {
                    spaces.push('\n');
                    self.line = self.line + 1;
                    self.ptr_next();
                }
                _ => {
                    return spaces;
                }
            }
        }
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

    #[allow(unused)]
    fn get_ident_or_soft(&mut self) -> String {
        self.remove_spaces();
        match self.pop() {
            Token::IDENT { str } => {
                return str;
            }
            tok => {
                let w = tok.try_get_soft_keyword();
                let w = match w {
                    Some(w) => w,
                    _ => return String::new(),
                };
                return w;
            }
        }
    }

    /// handles any mark found inside a body
    /// this is ONLY called coming from a body
    /// already consumed the mark
    /// ex: //-, /*- -*/ etc
    fn handle_mark_at_body(&mut self, body_str: &mut String, body: &mut Vec<Node>) -> Result<bool> {
        self.remove_spaces();
        match self.peek() {
            //- end:
            Token::END => {
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                *body_str = String::new();
                self.remove_spaces();
                match self.pop() {
                    // end :
                    Token::DD => {
                        self.remove_till_nl();
                        return Ok(true);
                    }
                    _ => return Err(parser_error(CompilationError::NoDDEndef, self)),
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
                                                return Err(parser_error(
                                                    CompilationError::NLArrowVarName,
                                                    self,
                                                ));
                                            }
                                            tok => {
                                                self.ptr_next();
                                                body.push(Node::RARROWVAR {
                                                    name,
                                                    default: Some(tok.val()),
                                                });
                                            }
                                        }
                                        return Ok(false);
                                    }
                                    _ => {
                                        return Err(parser_error(
                                            CompilationError::NotMarkAfterArrowVar,
                                            self,
                                        ));
                                    }
                                }
                            }
                            _ => return Err(parser_error(CompilationError::NotArrow, self)),
                        }
                    }
                    _ => return Err(parser_error(CompilationError::InvalidArrowVarName, self)),
                }
            }
            Token::DEF => {
                // inner def
                self.ptr_next();
                let mut nodes = ParsingResult::new(self.file_path.clone());
                self.handle_def(&mut nodes)?;
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
                self.handle_place(&mut nodes)?;
                body.append(&mut nodes.nodes);
                if matches!(self.peek(), Token::NL) {
                    self.line = self.line - 1;
                }
                //self.unpop();
            }
            Token::INCLUDE => {
                // inner include
                self.ptr_next();
                let mut nodes = ParsingResult::new(self.file_path.clone());
                self.handle_include(&mut nodes)?;
                body.append(&mut nodes.nodes);
            }
            Token::MATCH => {
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                *body_str = String::new();
                let node: Node = self.handle_match()?;
                body.push(node);
            }
            Token::FOR => {
                //- place a where var  = [(a,b),(b,c),(c,d)]
                //- for a,b in var:
                //- end:
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                *body_str = String::new();
                let node = self.handle_for()?;
                body.push(node);
            }
            Token::CREATE => {
                self.ptr_next();
                body.push(Node::DATA {
                    data: body_str.to_string(),
                    line: self.line,
                });
                *body_str = String::new();
                let node = self.handle_create()?;
                body.push(node);
            },
            _ => {
                return Err(parser_error(CompilationError::InvalidBodyCommand, self));
            }
        }
        return Ok(false);
    }

    // handles for loop
    // reaches here after for
    fn handle_for(&mut self) -> Result<Node> {
        let mut vars = vec![];
        // for a,b,c,d in ...
        loop {
            self.remove_spaces();
            match self.pop() {
                Token::IDENT { str } => {
                    vars.push(str);
                }
                tok => {
                    let kw = tok.try_get_soft_keyword();
                    match kw {
                        Some(w) => {
                            vars.push(w);
                        }
                        None => {
                            return Err(parser_error(CompilationError::InvalidVar, self));
                        }
                    }
                }
            }
            self.remove_spaces();
            match self.pop() {
                Token::COMMA => {
                    continue;
                }
                Token::IN => {
                    break;
                }
                _ => return Err(parser_error(CompilationError::InvalidVar, self)),
            }
        }
        // in var
        self.remove_spaces();
        let in_var = match self.pop() {
            Token::IDENT { str } => str,
            tok => {
                let w = tok.try_get_soft_keyword();
                match w {
                    Some(w) => w,
                    None => return Err(parser_error(CompilationError::InvalidVar, self)),
                }
            }
        };
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => return Err(parser_error(CompilationError::NoDDAfterQuotVar, self)),
        };
        self.remove_till_nl();
        let body = self.build_body()?;
        return Ok(Node::FOR {
            vars,
            in_var,
            body: Box::new(body),
        });
    }

   

    /// builds a body Node
    /// contains raw text and any nodes supported inside of a def body
    /// ends at "end"
    /// comes from def or match arm
    fn build_body(&mut self) -> Result<Node> {
        let mut body_str = String::new();
        let mut body: Vec<Node> = Vec::new();
        let line_start = self.line;
        loop {
            match self.peek() {
                // regular var declaration
                // $#var
                Token::VAR => {
                    self.ptr_next();
                    // push data before we continue
                    body.push(Node::DATA {
                        data: body_str.to_string(),
                        line: self.line,
                    });
                    body_str = String::new();
                    match self.peek() {
                        // $#ident
                        Token::IDENT { str } => {
                            self.ptr_next();
                            let mut option = None;
                            let mut optional = false;
                            match self.peek() {
                                Token::QUESTION => {
                                    self.ptr_next();
                                    optional = true;
                                }
                                _ => (),
                            }
                            match self.peek() {
                                Token::BSLASH => {
                                    self.ptr_next();
                                    option = self.handle_var_options()?;
                                }
                                _ => (),
                            }
                            match self.peek() {
                                Token::PLUS => {
                                    self.ptr_next();
                                }
                                _ => (),
                            }
                            body.push(Node::var_template(str, option, optional));
                            continue;
                        }
                        // $#name
                        _ => {
                            return Err(parser_error(CompilationError::InvalidVar, self));
                        }
                    }
                }
                Token::MARK { kind: _ } => {
                    self.ptr_next();
                    let should_break = self.handle_mark_at_body(&mut body_str, &mut body)?;
                    if should_break {
                        break;
                    }
                    continue;
                }
                Token::EOF => {
                    return Err(parser_error(CompilationError::BodyEOF, self));
                }
                Token::NL => {
                    self.ptr_next();
                    body_str.push_str("\n");
                    self.line = self.line + 1;
                }
                tok => {
                    self.ptr_next();
                    let val = &tok.val();
                    body_str.push_str(val);
                }
            }
        }
        return Ok(Node::BODY {
            data: body,
            line: line_start,
        });
    }
}
