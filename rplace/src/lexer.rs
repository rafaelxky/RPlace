use anyhow::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    IDENT { str: String },
    DEF,
    END,
    VAR,
    GLOBAL,
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
    LPAREN,
    RPAREN,
    DOT,
    PARSE,
    QUESTION,
    FOR,
    IN,
    MOD,
    PIPE,
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
            Token::LPAREN => "(",
            Token::RPAREN => ")",
            Token::DOT => ".",
            Token::PARSE => "parse",
            Token::QUESTION => "?",
            Token::FOR => "for",
            Token::IN => "in",
            Token::MOD => "mod",
            Token::PIPE => "here",
            Token::GLOBAL => "$g#",
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
            Token::PARSE => "parse",
            Token::FOR => "for",
            Token::IN => "in",
            Token::MOD => "mod",
            Token::PIPE => "here",
            _ => "",
        };
        if res.is_empty() {
            return None;
        } else {
            return Some(res.to_string());
        }
    }
    pub fn expect_dd_or_else(&self, predicate: fn() -> Error) -> Result<()> {
        match self {
            Token::DD => return Ok(()),
            _ => return Err(predicate()),
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
impl Lexer {
    pub fn new<T: ToString>(path: T, data: String) -> Self {
        let data = data.chars().collect();
        Self {
            ptr: 0,
            data,
            file_path: path.to_string(),
        }
    }
    fn peek(&self) -> Option<char> {
        self.data.get(self.ptr).copied()
    }
    fn pop(&mut self) -> char {
        self.ptr = self.ptr + 1;
        self.data[self.ptr - 1]
    }
    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pop();
            true
        } else {
            false
        }
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
                '?' => {
                    tokens.push(Token::QUESTION);
                    continue;
                }
                '.' => {
                    tokens.push(Token::DOT);
                    continue;
                }
                ':' => {
                    if self.consume_if(':') {
                        tokens.push(Token::QD);
                    } else {
                        tokens.push(Token::DD);
                    }
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
                }
                '(' => {
                    tokens.push(Token::LPAREN);
                    continue;
                }
                ')' => {
                    tokens.push(Token::RPAREN);
                    continue;
                }
                '\\' => {
                    tokens.push(Token::BSLASH);
                    continue;
                }
                '/' => {
                    if self.consume_if('/') {
                        if self.consume_if('-') {
                            tokens.push(Token::MARK {
                                kind: "//-".to_string(),
                            });
                        } else {
                            tokens.push(Token::IDENT {
                                str: "//".to_string(),
                            });
                        }
                    } else if self.consume_if('*') {
                        if self.consume_if('-') {
                            tokens.push(Token::MARK {
                                kind: "/*-".to_string(),
                            });
                        } else {
                            tokens.push(Token::IDENT {
                                str: "/*".to_string(),
                            });
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
                    let mut sequence = String::from("*");
                    let is_mark = if self.consume_if('/') {
                        sequence.push('/');
                        if self.consume_if('/') {
                            sequence.push('/');
                            if self.consume_if('/') {
                                sequence.push('/');
                                self.consume_if('-')
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if is_mark {
                        tokens.push(Token::MARK { kind: "*///-".to_string() });
                    } else {
                        tokens.push(Token::IDENT { str: sequence });
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
                    match self.data.get(self.ptr).copied() {
                        Some('#') => {
                            self.pop();
                            tokens.push(Token::VAR);
                        }
                        Some('g') if self.data.get(self.ptr + 1) == Some(&'#') => {
                            self.pop();
                            self.pop();
                            tokens.push(Token::GLOBAL);
                        }
                        _ => {
                            tokens.push(Token::IDENT {
                                str: "$".to_string(),
                            });
                        }
                    }
                    continue;
                }
                '-' => {
                    if self.consume_if('>') {
                        tokens.push(Token::RARROW);
                        continue;
                    } else if self.consume_if('*') {
                        if self.consume_if('/') {
                            tokens.push(Token::MARK {
                                kind: "-*/".to_string(),
                            });
                        } else {
                            tokens.push(Token::IDENT { str: "-*".to_string() });
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

            if is_char_terminator(curr) {
                tokens.push(Token::IDENT {
                    str: curr.to_string(),
                });
                continue;
            }

            let mut str = String::from(curr);
            while self.can_pop() {
                match self.peek() {
                    Some(next) if is_char_terminator(next) => break,
                    Some(_) => str.push(self.pop()),
                    None => break,
                }
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
                }
                "create" => {
                    tokens.push(Token::CREATE);
                    continue;
                }
                "derive" => {
                    tokens.push(Token::DERIVE);
                    continue;
                }
                "case" => {
                    tokens.push(Token::CASE);
                    continue;
                }
                "match" => {
                    tokens.push(Token::MATCH);
                    continue;
                }
                "parse" => {
                    tokens.push(Token::PARSE);
                    continue;
                }
                "for" => {
                    tokens.push(Token::FOR);
                    continue;
                }
                "in" => {
                    tokens.push(Token::IN);
                    continue;
                }
                "mod" => {
                    tokens.push(Token::MOD);
                    continue;
                }
                "here" => {
                    tokens.push(Token::PIPE);
                    continue;
                }
                _ => {
                    tokens.push(Token::IDENT { str });
                    continue;
                }
            }
        }

        //self.handle_interruption_tokens(&mut tokens);
        //return tokens;
    }
}
fn is_char_terminator(char: char) -> bool {
    match char {
        '_' => return false,
        _ => (),
    }
    if !char.is_alphanumeric() {
        return true;
    }
    false
}
