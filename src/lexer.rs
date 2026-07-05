
#[derive(Debug, Clone)]
pub enum Token {
    IDENT { str: String },
    DEF,
    END,
    VAR,
    MARK { kind: String },
    DD,
    EOF,
    PLACE,
    WHERE,
    SPACE,
    NL,
    EQUALS,
    COMMA,
    LSRQBRACK,
    RSRQBRACK,
    DQUOTE,
    INCLUDE,
    RARROW,
    WHEN,
    PLUS,
    CREATE,
    BSLASH,
    DERIVE,
    CASE,
    MATCH,
    QD,
}
impl Token {
    pub fn val(&self) -> String {
        return match self {
            Token::IDENT { str } => str,
            Token::DEF => "def",
            Token::END => "end",
            Token::VAR => "$#",
            Token::MARK { kind } => kind,
            Token::DD => ":",
            Token::EOF => "",
            Token::PLACE => "place",
            Token::WHERE => "where",
            Token::SPACE => " ",
            Token::NL => "\n",
            Token::EQUALS => "=",
            Token::COMMA => ",",
            Token::LSRQBRACK => "[",
            Token::RSRQBRACK => "]",
            Token::DQUOTE => "\"",
            Token::INCLUDE => "include",
            Token::RARROW => "->",
            Token::WHEN => "when",
            Token::PLUS => "+",
            Token::CREATE => "create",
            Token::BSLASH => "\\",
            Token::DERIVE => "derive",
            Token::CASE => "case",
            Token::MATCH => "match",
            Token::QD => "::",
        }
        .to_string();
    }
    pub fn try_get_soft_keyword(&self) -> Option<String> {
        let res = match self {
            Token::DEF => "def",
            Token::END => "endef",
            Token::PLACE => "place",
            Token::WHERE => "where",
            Token::INCLUDE => "include",
            Token::WHEN => "when",
            Token::CREATE => "create",
            Token::DERIVE => "derive",
            Token::CASE => "case",
            Token::MATCH => "match",
            _ => ""
        };
        if res.is_empty() {
            return None;
        } else {
            return Some(res.to_string());
        }
    }
}
pub struct TokenResult {
    pub tokens: Vec<Token>,
    pub file_path: String,
}
pub struct Lexer {
    ptr: usize,
    data: Vec<char>,
    file_path: String,
}
// this is all wrong, correctness is in parser not lexer
impl Lexer {
    pub fn new<T: ToString>(path: T, data: String) -> Self {
        let data = data.chars().collect();
        Self {
            ptr: 0,
            data,
            file_path: path.to_string(),
        }
    }
    fn peek(&self) -> char {
        self.data[self.ptr]
    }
    fn pop(&mut self) -> char {
        self.ptr = self.ptr + 1;
        self.data[self.ptr - 1]
    }
    fn unpop(&mut self) {
        self.ptr = self.ptr - 1;
    }
    fn can_pop(&self) -> bool {
        self.ptr < self.data.len()
    }

    pub fn parse(mut self) -> TokenResult {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if !self.can_pop() {
                tokens.push(Token::EOF);
                return TokenResult {
                    tokens: tokens,
                    file_path: self.file_path,
                };
            }
            let curr = self.pop();

            match curr {
                ':' => {
                    if !self.can_pop() {
                        tokens.push(Token::DD);
                        continue;
                    }
                    match self.peek() {
                        ':' => {
                            self.pop();
                            tokens.push(Token::QD);
                            continue;
                        },
                        _ => (),
                    }
                    tokens.push(Token::DD);
                    continue;
                }
                '=' => {
                    tokens.push(Token::EQUALS);
                    continue;
                }
                ',' => {
                    tokens.push(Token::COMMA);
                    continue;
                }
                '[' => {
                    tokens.push(Token::LSRQBRACK);
                    continue;
                }
                ']' => {
                    tokens.push(Token::RSRQBRACK);
                    continue;
                }
                '+' => {
                    tokens.push(Token::PLUS);
                    continue;
                }
                '"' => {
                    tokens.push(Token::DQUOTE);
                    continue;
                },
                '\\' => {
                    tokens.push(Token::BSLASH);
                    continue;
                },
                '/' => {
                    // //-
                    if self.peek() == '/' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK {
                                kind: "//-".to_string(),
                            });
                        }
                    } else
                    // /*-
                    if self.peek() == '*' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK {
                                kind: "/*-".to_string(),
                            });
                        } else {
                            tokens.push(Token::IDENT {
                                str: "/*".to_string(),
                            })
                        }
                    } else {
                        tokens.push(Token::IDENT {
                            str: "/".to_string(),
                        });
                    }
                    continue;
                }
                // *///-
                '*' => {
                    if self.peek() == '/' {
                        self.pop();
                        if self.peek() == '/' {
                            self.pop();
                            if self.peek() == '/' {
                                self.pop();
                                if self.peek() == '-' {
                                    self.pop();
                                    tokens.push(Token::MARK {
                                        kind: "*///-".to_string(),
                                    });
                                } else {
                                    tokens.push(Token::IDENT {
                                        str: "*///".to_string(),
                                    });
                                }
                            } else {
                                tokens.push(Token::IDENT {
                                    str: "*//".to_string(),
                                });
                            }
                        } else {
                            tokens.push(Token::IDENT {
                                str: "*/".to_string(),
                            });
                        }
                    } else {
                        tokens.push(Token::IDENT {
                            str: "*".to_string(),
                        });
                    }
                    continue;
                }
                ' ' => {
                    tokens.push(Token::SPACE);
                    continue;
                }
                '\n' => {
                    tokens.push(Token::NL);
                    continue;
                }
                '$' => {
                    if self.peek() == '#' {
                        self.pop();
                        tokens.push(Token::VAR);
                    } else {
                        tokens.push(Token::IDENT {
                            str: "$".to_string(),
                        });
                    }
                    continue;
                }
                '-' => {
                    if self.peek() == '>' {
                        self.pop();
                        tokens.push(Token::RARROW);
                        continue;
                    } else if self.peek() == '*' {
                        self.pop();
                        if self.peek() == '/' {
                            self.pop();
                            tokens.push(Token::MARK {
                                kind: "-*/".to_string(),
                            });
                        } else {
                            tokens.push(Token::IDENT {
                                str: "-/".to_string(),
                            });
                        }
                        continue;
                    } else {
                        tokens.push(Token::IDENT {
                            str: "-".to_string(),
                        });
                        continue;
                    }
                }
                _ => (),
            }

            let mut str = String::new();

            if self.is_char_terminator(curr) {
                tokens.push(Token::IDENT {
                    str: curr.to_string(),
                });
                continue;
            }

            self.unpop();
            while self.can_pop() {
                if self.is_char_terminator(self.peek()) {
                    break;
                }
                str.push(self.pop());
            }

            match str.as_str() {
                "place" => {
                    tokens.push(Token::PLACE);
                    continue;
                }
                "def" => {
                    tokens.push(Token::DEF);
                    continue;
                }
                "end" => {
                    tokens.push(Token::END);
                    continue;
                }
                "where" => {
                    tokens.push(Token::WHERE);
                    continue;
                }
                "include" => {
                    tokens.push(Token::INCLUDE);
                    continue;
                }
                "when" => {
                    tokens.push(Token::WHEN);
                    continue;
                },
                "create" => {
                    tokens.push(Token::CREATE);
                },
                "derive" => {
                    tokens.push(Token::DERIVE);
                },
                "case" => {
                    tokens.push(Token::CASE);
                },
                "match" => {
                    tokens.push(Token::MATCH);
                },
                _ => {
                    tokens.push(Token::IDENT { str });
                    continue;
                }
            }
        }

        //self.handle_interruption_tokens(&mut tokens);
        //return tokens;
    }

    fn is_char_terminator(&self, char: char) -> bool {
        match char {
            '_' => return false,
            _ => (),
        }
        if !char.is_alphanumeric() {
            return true;
        }
        false
    }
}
