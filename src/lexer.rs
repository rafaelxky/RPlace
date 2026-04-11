use core::panic;
use std::{fs, path::Path, str::Chars, thread::current};

#[derive(Debug, Clone)]
pub enum Token {
    IDENT { str: String },
    DEF,
    ENDEF,
    VAR,
    MARK,
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
}
pub struct Lexer {
    ptr: usize,
    data: Vec<char>,
}
// this is all wrong, correctness is in parser not lexer
impl Lexer {
    pub fn new<T: ToString>(path: T) -> Self {
        if !Path::new(&path.to_string()).exists() {
            panic!("Couldnt find file {}", path.to_string());
        }
        let data = fs::read_to_string(path.to_string())
            .unwrap()
            .chars()
            .collect();
        Self { ptr: 0, data }
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
    fn peek_ahead(&self, i: usize) -> char {
        self.data[self.ptr + i]
    }
    fn can_pop(&self) -> bool {
        self.ptr < self.data.len()
    }

    pub fn parse(mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            if !self.can_pop() {
                tokens.push(Token::EOF);
                println!("eof");
                return tokens;
            }

            let mut curr = self.pop();

            match curr {
                ':' => {
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
                '"' => {
                    tokens.push(Token::DQUOTE);
                    continue;
                }
                '/' => {
                    // //-
                    if self.peek() == '/' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
                        }
                    } else
                    // /*-
                    if self.peek() == '*' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
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
                                    tokens.push(Token::MARK);
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
                _ => (),
            }

            let mut str = String::new();

            if self.is_char_terminator(curr) {
                tokens.push(Token::IDENT {
                    str: curr.to_string(),
                });
                continue;
            }

            while self.can_pop() && !self.is_char_terminator(curr) {
                str.push(curr);
                curr = self.pop();
            }
            self.unpop();

            match str.as_str() {
                "place" => {
                    tokens.push(Token::PLACE);
                    continue;
                }
                "def" => {
                    tokens.push(Token::DEF);
                    continue;
                }
                "endef" => {
                    tokens.push(Token::ENDEF);
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
