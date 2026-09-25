use anyhow::{Ok, Result};

use crate::{
    error_handler::{parser_error, CompilationError},
    lexer::Token,
    parser::Parser,
    structs::{MatchArm, Node, ParsingResult, Value, Var},
};

impl Parser {
    // gets here after //- $#
    // currently only handles global variable set for stuff like file config
    // ex: //- $#var = val:
    pub(super) fn handle_set_variable(&mut self, nodes: &mut ParsingResult) -> Result<()>{
        let mut var = vec![];
        loop {
            let var_name = match self.pop() {
                Token::IDENT { str } => str,
                _ => return Err(parser_error(CompilationError::InvalidVar, self)),
            };
            var.push(var_name);
            match self.peek() {
                Token::DOT => {
                    self.ptr_next();
                }
                _tok => break,
            }
        }
        self.remove_spaces();
        match self.pop() {
            Token::EQUALS => {}
            _ => return Err(parser_error(CompilationError::InvalidAssignementDefWhere, self)),
        }
        self.remove_spaces();

        let val = self.handle_val()?;
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => return Err(parser_error(CompilationError::NoDDAfterQuotVar, self)),
        }

        nodes.push(Node::SETVARIABLE { var: var, val: val });
        Ok(())
    }
    /// //- parse file.txt:
    pub(super) fn handle_parse_instr(&mut self, nodes: &mut ParsingResult) -> Result<()> {
        let path = self.handle_path(self.project_src.clone())?;
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => return Err(parser_error(CompilationError::InvalidAfterFilePath, self)),
        };
        nodes.push(Node::PARSE { path });
        Ok(())
    }

    // //- derive file.txt:
    pub(super) fn handle_derive(&mut self, nodes: &mut ParsingResult) -> Result<()>{
        self.remove_spaces();
        let path = match self.peek() {
            Token::IDENT { str: _ } => self.handle_path(self.project_src.clone())?,
            _ => self.file_path.to_string(),
        };
        self.remove_spaces();

        // derive options
        let args: Vec<(Var, Value)> = match self.peek() {
            Token::WHERE => {
                self.ptr_next();
                self.remove_spaces();
                let args = self.handle_vars()?;
                match self.peek() {
                    Token::DD => {
                        self.ptr_next();
                        args
                    }
                    _ => return Err(parser_error(CompilationError::InvalidDeriveOption, self)),
                }
            }
            _ => return Err(parser_error(CompilationError::InvalidDeriveOption, self)),
        };
        nodes.push(Node::DERIVE {
            path: path,
            val: args,
        });
        Ok(())
    }

    // create filepath place defname:
    // reaches here after 
    // todo: allow create with content other than place
    // todo: allow //- create $#path
    pub(super) fn handle_create(&mut self) -> Result<Node>{
        let path: String = self.handle_path(self.output_src.clone())?;
        let starting_line = self.get_line();
        // filepath
        // ex: parent/child.txt

        self.remove_spaces();

        match self.peek() {
            Token::DD => {
                self.ptr_next();
                let node = Node::CREATE { path, content: None };
                self.remove_till_nl();
                return Ok(node);
            }
            Token::PLACE => {
                self.ptr_next();
                let mut temp_nodes = ParsingResult::new(path.clone());
                // returns one place
                self.handle_place(&mut temp_nodes)?;
                //let content = Some(Box::new(Node::BODY { data: temp_nodes, line: starting_line }))
                let node = Node::new_create(path, temp_nodes.nodes, starting_line);
                return Ok(node);
            }
            _ => return Err(parser_error(CompilationError::InvalidAfterFilePath, self)),
        }
    }

    pub(super) fn handle_place(&mut self, nodes: &mut ParsingResult) -> Result<()>{
        // reaches here as //- place
        self.remove_spaces();

        let place_id = match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                str
            }
            tok => {
                self.ptr_next();
                let w = tok.try_get_soft_keyword();
                match w {
                    Some(w) => w,
                    None => return Err(parser_error(CompilationError::InvalidPlaceName, self)),
                }
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
                    args.append(&mut self.handle_vars()?);
                    self.remove_spaces();
                }
                _ => return Err(parser_error(CompilationError::InvalidPlaceOption, self)),
            }
        }
        nodes.push(Node::PLACE {
            name: place_id,
            args: args,
            line: place_line,
        });
        return Ok(());
    }

    // //- match var:
    pub(super) fn handle_match(&mut self) -> Result<Node> {
        self.remove_till_nl();
        let var_name = match self.pop() {
            Token::IDENT { str } => str,
            _ => return Err(parser_error(CompilationError::Invalid1stIdentWhen, self)),
        };
        self.remove_till_nl();
        match self.pop() {
            Token::DD => {}
            _ => return Err(parser_error(CompilationError::NoDDAfterQuotVar, self)),
        };
        let mut matches: Vec<MatchArm> = Vec::new();
        loop {
            self.remove_spaces();
            match self.pop() {
                Token::MARK { kind: _ } => {}
                _ => return Err(parser_error(CompilationError::InvalidBodyCommand, self)),
            }
            self.remove_spaces();
            match self.pop() {
                Token::CASE => {
                    let arm_body = self.handle_match_arm()?;
                    matches.push(arm_body);
                }
                Token::END => {
                    self.ptr_next();
                    break;
                }
                _ => return Err(parser_error(CompilationError::InvalidBodyCommand, self)),
            }
        }

        return Ok(Node::MATCH {
            line: self.line,
            var_name: var_name,
            val: matches,
        });
    }
    /// handles match arm
    /// already poped <case> token here
    /// returns a body node and the match value inside the match arm struct
    /// ex: //- case name: nody //- end:
    pub(super) fn handle_match_arm(&mut self) -> Result<MatchArm> {
        self.remove_spaces();
        // case name
        let match_value = match self.pop() {
            Token::IDENT { str } => str,
            _ => return Err(parser_error(CompilationError::Invalid2ndIdentWhen, self)),
        };

        self.remove_spaces();
        match self.pop() {
            Token::DD => {
                self.remove_till_nl();
            }
            _ => return Err(parser_error(CompilationError::NoDDAfterQuotVar, self)),
        };

        let body = self.build_body()?;
        Ok(MatchArm::new(match_value, body))
    }

    // //- include text.txt:
    pub(super) fn handle_include(&mut self, nodes: &mut ParsingResult) -> Result<()> {
        self.remove_spaces();

        let path = match self.peek() {
            Token::IDENT { str: _ } => self.handle_path(self.project_src.clone())?,
            _ => {
                return Err(parser_error(CompilationError::InvalidTokenInIncludePath, self));
            }
        };

        self.remove_till_nl();

        match self.pop() {
            Token::DD => {}
            _ => {
                return Err(parser_error(CompilationError::NoDDAfterQuotVar, self));
            }
        }

        nodes.push(Node::INCLUDE {
            path: path.clone(),
            line: self.line,
        });

        Ok(())
    }

    // //- mod file.txt:
    pub (super) fn handle_mod(&mut self, nodes: &mut ParsingResult) -> Result<()> {
        self.remove_spaces();
        let path = self.handle_path(self.file_path.clone())?;
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => return Err(parser_error(CompilationError::NoDDAfterQuotVar, self)),
        }
        nodes.push(Node::MOD { path });
        Ok(())
    }   

}
