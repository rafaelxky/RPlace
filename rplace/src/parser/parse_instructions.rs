use crate::{
    error_handler::{CompilationError, handle_error_parser},
    lexer::Token,
    parser::Parser,
    structs::{MatchArm, Node, ParsingResult, Value, Var},
};

impl Parser {
    // gets here after //- $#
    // currently only handles global variable set for stuff like file config
    // ex: //- $#var = val:
    pub(super) fn handle_set_variable(&mut self, nodes: &mut ParsingResult) {
        let mut var = vec![];
        loop {
            let var_name = match self.pop() {
                Token::IDENT { str } => str,
                tok => panic!("Expected Ident found {:?}", tok),
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
            tok => panic!("Expected = found {:?}", tok),
        }
        self.remove_spaces();

        let val = self.handle_val();
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => panic!("todo message: forgot :"),
        }

        nodes.push(Node::SETVARIABLE { var: var, val: val });
    }
    /// //- parse file.txt:
    pub(super) fn handle_parse_instr(&mut self, nodes: &mut ParsingResult) {
        let path = self.handle_path(self.project_src.clone());
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => panic!("todo message: forgot : at PARSE"),
        };
        nodes.push(Node::PARSE { path });
    }

    // //- derive file.txt:
    pub(super) fn handle_derive(&mut self, nodes: &mut ParsingResult) {
        self.remove_spaces();
        let path = match self.peek() {
            Token::IDENT { str: _ } => self.handle_path(self.project_src.clone()),
            _ => self.file_path.to_string(),
        };
        self.remove_spaces();

        // derive options
        let args: Vec<(Var, Value)> = match self.peek() {
            Token::WHERE => {
                self.ptr_next();
                self.remove_spaces();
                let args = self.handle_vars();
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
    pub(super) fn handle_create(&mut self, nodes: &mut ParsingResult) {
        let path: String = self.handle_path(self.output_src.clone());
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

    pub(super) fn handle_place(&mut self, nodes: &mut ParsingResult) {
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
                    None => handle_error_parser(CompilationError::InvalidPlaceName, self),
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
                    args.append(&mut self.handle_vars());
                    self.remove_spaces();
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

    // //- match var:
    pub(super) fn handle_match(&mut self) -> Node {
        self.remove_till_nl();
        let var_name = match self.pop() {
            Token::IDENT { str } => str,
            _ => panic!("todo error message expected ident in match"),
        };
        self.remove_till_nl();
        match self.pop() {
            Token::DD => {}
            _ => panic!("todo error message expected : in match"),
        };
        let mut matches = Vec::new();
        loop {
            self.remove_spaces();
            match self.pop() {
                Token::MARK { kind: _ } => {}
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
    /// already poped <case> token here
    /// returns a body node and the match value inside the match arm struct
    /// ex: //- case name: nody //- end:
    pub(super) fn handle_match_arm(&mut self) -> MatchArm {
        self.remove_spaces();
        // case name
        let match_value = match self.pop() {
            Token::IDENT { str } => str,
            _ => panic!("todo error message expected ident at match arm"),
        };

        self.remove_spaces();
        match self.pop() {
            Token::DD => {
                self.remove_till_nl();
            }
            _ => panic!("todo error message expected : at match arm"),
        };

        let body = self.build_body();
        MatchArm::new(match_value, body)
    }

    // //- include text.txt:
    pub(super) fn handle_include(&mut self, nodes: &mut ParsingResult) {
        self.remove_spaces();

        let path = match self.peek() {
            Token::IDENT { str: _ } => self.handle_path(self.project_src.clone()),
            _ => {
                handle_error_parser(CompilationError::InvalidTokenInIncludePath, self);
            }
        };

        self.remove_till_nl();

        match self.pop() {
            Token::DD => {}
            _ => {
                panic!("todo error message, expected :")
            }
        }

        nodes.push(Node::INCLUDE {
            path: path.clone(),
            line: self.line,
        });

        return;
    }

    // //- mod file.txt:
    pub (super) fn handle_mod(&mut self, nodes: &mut ParsingResult){
        self.remove_spaces();
        let path = self.handle_path(self.file_path.clone());
        self.remove_spaces();
        match self.pop() {
            Token::DD => (),
            _ => panic!("todo message expected : at mod")
        }
        nodes.push(Node::MOD { path });
    }   

}
