use std::path::Path;

use path_clean::PathClean;

use crate::{
    error_handler::{CompilationError, handle_error, handle_error_parser}, lexer::Token, parser::Parser, structs::{Value, Var, VarOption},
};

impl Parser {
    /// handles values
    /// reaches here after =
    /// single word values
    /// double quote values
    /// multiline quote values
    pub(super) fn handle_val(&mut self) -> Value {
        let mut options = None;
        match self.peek() {
            // ident = ident -> variable assignement
            Token::IDENT { str } => {
                self.ptr_next();
                if matches!(self.peek(), Token::BSLASH) {
                    self.ptr_next();
                    options = self.handle_var_options();
                }
                self.remove_spaces();
                return Value::new_literal_type(str, options);
            }
            // ident = "ident" -> quotation handling for multiline values
            Token::DQUOTE => {
                self.ptr_next();
                let mut args = vec![];
                self.get_dquote_arg_var(&mut options, &mut args, &mut "".to_string());
                return args[0].clone().1;
            }
            // $#var
            Token::VAR => {
                self.ptr_next();
                match self.peek() {
                    Token::IDENT { str } => {
                        self.ptr_next();
                        if matches!(self.peek(), Token::BSLASH) {
                            self.ptr_next();
                            options = self.handle_var_options();
                        }
                        self.remove_spaces();
                        return Value::new_var_type(str, options);
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
            Token::LSRQBRACK => {
                self.ptr_next();
                return self.handle_array_values();
            }
            _ => handle_error_parser(CompilationError::Invalid2ndPlaceVar, self),
        }
    }

    // the oneshot version of handle_vars
    // expects ident = ident
    // ends after that
    pub(super) fn handle_var(&mut self) -> (Var, Value) {
        self.remove_spaces();
        // name
        let var_name = match self.peek() {
            Token::IDENT { str } => {
                self.ptr_next();
                str
            }
            _ => handle_error_parser(CompilationError::Invalid1stPlaceVar, self),
        };
        self.remove_spaces();
        // =
        match self.peek() {
            Token::EQUALS => self.ptr_next(),
            _ => panic!("todo message"),
        };
        self.remove_spaces();
        // value
        let arg = self.handle_val();
        (Var::new(var_name), arg)
    }

    // here after anything that requires variable assignement
    // ex: before this -> name = val
    // handles the whole var = val, var = val
    // doesn't consume the final :
    pub(super) fn handle_vars(&mut self) -> Vec<(Var, Value)> {
        let mut args: Vec<(Var, Value)> = Vec::new();
        loop {
            self.remove_spaces();
            let arg = self.handle_var();
            args.push(arg);
            self.remove_spaces();
            match self.peek() {
                Token::COMMA => {
                    self.ptr_next();
                    continue;
                }
                Token::DD => {
                    return args;
                }
                t => {
                    panic!("todo message: Unexpected token {:?} in handle vars", t)
                }
            }
        }
    }

    // reaches here after [
    // ends at ] (consumes it)
    // ex: [(a,b),(c,d)]
    pub(super) fn handle_array_values(&mut self) -> Value {
        let mut vals: Vec<Vec<Value>> = vec![];
        let mut names: Vec<Vec<Option<String>>> = vec![];
        loop {
            self.remove_spaces();
            match self.pop() {
                Token::LPAREN => {
                    vals.push(vec![]);
                    loop {
                        let val = self.handle_val();
                        let len = vals.len() - 1;
                        vals[len].push(val);
                        self.remove_spaces();
                        match self.pop() {
                            Token::COMMA => continue,
                            Token::RPAREN => break,
                            _ => panic!(),
                        }
                    }
                }
                tok => panic!("todo error message, expected lparen, found {:?}", tok),
            }
            self.remove_spaces();
            match self.pop() {
                Token::COMMA => continue,
                Token::RSRQBRACK => break,
                _ => panic!("todo"),
            }
        }

        // todo: names
        // [(name=val, name2=val2)]
        return Value::new_array_type(vals, names);
    }

    /// gets here right after " in arg
    pub(super) fn get_dquote_arg_var(
        &mut self,
        options_2: &mut Option<Vec<VarOption>>,
        args: &mut Vec<(String, Value)>,
        from: &mut String,
    ) {
        let arg_str = self.get_dquote_var();

        if matches!(self.peek(), Token::BSLASH) {
            self.ptr_next();
            *options_2 = self.handle_var_options();
        }
        self.remove_spaces();
        args.push((
            from.to_string(),
            Value::new_literal_type(arg_str, options_2.clone()),
        ));
    }

    /// gets the file path from tokens
    /// at this point we know a path is to come but no ident has been consumed
    /// stops at space or :
    /// does not consume :
    /// ex: parent/child.txt
    pub(super) fn handle_path(&mut self, base_path: String) -> String {
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
                Token::DOT => {
                    self.ptr_next();
                    path.push('.');
                }
                _ => handle_error_parser(CompilationError::InvalidTokenInPath, self),
            }
        }
        if path.starts_with('.') {
            let path_inner = path.strip_prefix("./").unwrap();
            let mut new_path = Path::new(&base_path).parent().unwrap().to_path_buf();
            new_path.push(path_inner);
            path = new_path.to_string_lossy().to_string();
        };

        path = Path::new(&path).clean().to_str().unwrap().to_string();
        let root = Path::new(&base_path).clean();
        let root = root.parent().unwrap();
        let root = root.to_str().unwrap();
        if !path.starts_with(&root) {
            panic!(
                "todo message: path escapes project root {} is outside of {}",
                path, root
            );
        }

        path
    }

    /// handles variable options
    /// reaches here at the ident after \
    /// returns a list of the options
    /// ex: $#var\CAMEL
    pub(super) fn handle_var_options(&mut self) -> Option<Vec<VarOption>> {
        let mut options: Option<Vec<VarOption>> = None;
        'outer: loop {
            match self.pop() {
                Token::IDENT { str } => {
                    if options.is_none() {
                        options = Some(Vec::new());
                    }
                    options.as_mut().unwrap().push(VarOption::new(str, vec![]));
                    'inner: loop {
                        match self.peek() {
                            Token::QD => {
                                self.ptr_next();
                                match self.peek() {
                                    Token::IDENT { str } => {
                                        self.ptr_next();
                                        if let Some(options) = options.as_mut() {
                                            if let Some(last) = options.last_mut() {
                                                last.push_arg(str);
                                            }
                                        }
                                    }
                                    Token::DQUOTE => {
                                        self.ptr_next();
                                        if let Some(options) = options.as_mut() {
                                            if let Some(last) = options.last_mut() {
                                                let str = self.get_dquote_var();
                                                last.push_arg(str);
                                            }
                                        }
                                    }
                                    _ => panic!("todo panic msg {:?}", self.peek()),
                                }
                            }
                            Token::BSLASH => {
                                self.ptr_next();
                                break 'inner;
                            }
                            _ => break 'outer,
                        }
                    }
                }
                tok => match tok.try_get_soft_keyword() {
                    Some(str) => {
                        if options.is_none() {
                            options = Some(Vec::new());
                        }
                        options.as_mut().unwrap().push(VarOption::new(str, vec![]));
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

    // gets here after the first "
    // ends at the second "
    pub(super) fn get_dquote_var(&mut self) -> String {
        let mut arg_str = String::new();
        let mut has_new_line = false;

        loop {
            if !self.can_pop() {
                panic!("todo msg. found eof at dqote var")
            }
            match self.peek() {
                Token::NL => {
                    arg_str.push('\n');
                    self.line = self.line + 1;
                    has_new_line = true;
                    self.ptr_next();
                }
                // \"
                Token::BSLASH => {
                    self.ptr_next();
                    match self.peek() {
                        Token::DQUOTE => {
                            self.ptr_next();
                            arg_str.push('"');
                        }
                        _ => {
                            arg_str.push('\\');
                        }
                    }
                }
                Token::DQUOTE => {
                    self.ptr_next();
                    if has_new_line {
                        arg_str.push('"');
                    } else {
                        break;
                    }
                }
                Token::MARK { kind } => {
                    self.ptr_next();
                    if !has_new_line {
                        arg_str.push_str(&kind);
                    } else {
                        // if value has a newline after the first ", then ends at mark + "
                        // //-" ends the multiline dquote val
                        let spaces = self.remove_and_return_spaces();
                        arg_str.push_str(&spaces);
                        match self.peek() {
                            Token::DQUOTE => {
                                self.ptr_next();
                                break;
                            }
                            _ => (),
                        }
                        // if has " after mark
                    }
                }
                tok => {
                    self.ptr_next();
                    arg_str.push_str(&tok.val());
                }
            }
        }
        arg_str
    }


}
