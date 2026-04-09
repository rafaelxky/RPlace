use core::panic;
use std::{fs, str::Chars, thread::current};

#[derive(Debug, Clone)]
pub enum Token {
    LPAREN,
    RPAREN,
    HASH,
    IDENT { str: String },
    DEF,
    ENDEF,
    VAR { name: String },
    MARK,
    DD,
    EOF,
}
pub struct Lexer {
    ptr: usize,
    data: Vec<char>,
}
impl Lexer {
    pub fn new<T: ToString>(path: T) -> Self {
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
    fn peek_ahead(&self, i: usize) -> char {
        self.data[self.ptr + i]
    }
    fn can_pop(&self) -> bool {
        self.ptr < self.data.len()
    }
    pub fn parse(mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        // termination tokens
        // //
        // #
        // (
        // )
        // \n
        // space

        // single char tokens
        self.handle_interruption_tokens(&mut tokens);
        return tokens;
    }

    fn handle_ident(&mut self, char: char, tokens: &mut Vec<Token>) {
        let mut str = String::new();
        // while not terminator, build ident
        str.push(char);

        while self.can_pop() && !self.is_char_terminator(self.peek()) {
            let curr = self.pop();
            str.push(curr);
        }
        match str.as_str() {
            "def" => {
                tokens.push(Token::DEF);
            }
            "endef" => {
                tokens.push(Token::ENDEF);
            }
            _ => {
                tokens.push(Token::IDENT { str });
            }
        }
    }

    fn is_char_terminator(&self, char: char) -> bool {
        match char {
            ' ' => true,
            '(' => true,
            ')' => true,
            '\n' => true,
            '/' => true,
            '-' => true,
            '$' => true,
            '#' => true,
            ':' => true,
            '*' => true,
            ',' => true,
            '{' => true,
            '}' => true,
            '[' => true,
            ']' => true,
            '.' => true,
            _ => false,
        }
    }

    fn handle_interruption_tokens(&mut self, tokens: &mut Vec<Token>) {
        while self.can_pop() {
            let current = self.pop();
            match current {
                ' ' => {
                    continue;
                }
                '(' => {
                    tokens.push(Token::LPAREN);
                }
                ')' => {
                    tokens.push(Token::RPAREN);
                }
                '\n' => {
                    continue;
                }
                '#' => {
                    tokens.push(Token::HASH);
                }
                '/' => {
                    if self.peek() == '/' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
                        } else {
                            tokens.push(Token::IDENT {
                                str: "//".to_string(),
                            });
                        }
                    } else if self.peek() == '*' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
                            self.handle_instruction(tokens);
                            self.handle_code_block(tokens);
                        } else {
                            tokens.push(Token::IDENT {
                                str: "/*".to_string(),
                            });
                        }
                    } else {
                        break;
                    }
                }
                '*' => {
                    if self.pop() == '/' {

                        if self.pop() != '/' {
                            tokens.push(Token::IDENT {
                                str: "*/".to_string(),
                            });
                        }

                        if self.pop() != '/' {
                            tokens.push(Token::IDENT {
                                str: "*//".to_string(),
                            });
                        }

                        if self.pop() == '-' {
                            tokens.push(Token::MARK);
                        } else {
                            tokens.push(Token::IDENT {
                                str: "*///".to_string(),
                            });
                        }
                    } else {
                        tokens.push(Token::IDENT {
                            str: "*".to_string(),
                        });
                    }
                }
                ':' => {
                    tokens.push(Token::DD);
                }
                '$' => {
                    if self.peek() == '#' {
                        self.pop();
                        self.handle_var(tokens);
                    } else {
                        tokens.push(Token::IDENT {
                            str: '$'.to_string(),
                        });
                    }
                }
                _ => {
                    self.handle_ident(current, tokens);
                }
            }
        }
    }
    fn handle_var(&mut self, tokens: &mut Vec<Token>) {
        let mut var_name = String::new();
        while self.can_pop() && !self.is_char_terminator(self.peek()) {
            let char = self.pop();
            var_name.push(char);
        }

        tokens.push(Token::VAR { name: var_name });
    }

    fn handle_instruction(&mut self, tokens: &mut Vec<Token>) {
        let mut instr = String::new();
        let mut curr = self.peek();
        while curr == ' ' {
            curr = self.pop();
        }
        while !self.is_char_terminator(curr) {
            instr.push(curr);
            curr = self.pop();
        }

        match instr.as_str() {
            "def" => {
                tokens.push(Token::DEF);
            }
            _ => {
                panic!("{} is an invalid instruction!", instr);
            }
        }

        curr = self.pop();
        self.handle_ident(curr, tokens);

        while curr == ' ' {
            curr = self.pop();
        }
        if self.peek() == ':' {
            tokens.push(Token::DD);
            self.pop();
        } else {
            // todo:
        }
    }

    fn handle_code_block(&mut self, tokens: &mut Vec<Token>) {
        let mut curr = self.pop();
        let mut str = String::new();
        loop {
            if !self.can_pop() {
                return;
            }
            match curr {
                '*' => {
                    let next = self.peek();
                    match next {
                        '/' => {
                            self.pop();
                            if self.pop() != '/' {
                                str.push_str("*/");
                            }
                            if self.pop() != '/' {
                                str.push_str("*//");
                            }

                            let next2 = self.peek();
                            // *///-
                            match next2 {
                                '-' => {
                                    self.pop();
                                    tokens.push(Token::IDENT { str: str.clone() });
                                    tokens.push(Token::MARK);
                                    let mut ident_str = String::new();
                                    let mut curr_isntr_char = self.pop();

                                    while curr_isntr_char == ' ' {
                                        curr_isntr_char = self.pop();
                                    }
                                    while !self.is_char_terminator(curr_isntr_char) {
                                        ident_str.push(curr_isntr_char);
                                        curr_isntr_char = self.pop();
                                    }
                                    match ident_str.as_str() {
                                        "endef" => {
                                            tokens.push(Token::ENDEF);
                                            if curr_isntr_char == ':' {
                                                tokens.push(Token::DD);
                                                return;
                                            } else {
                                            }
                                        }
                                        _ => {
                                            panic!("{} is not a valid instruction!", ident_str);
                                        }
                                    }
                                }
                                _ => {
                                    str.push_str("*///");
                                }
                            }
                        }
                        _ => {
                            str.push(curr);
                        }
                    }
                }
                '/' => {}
                '$' => {
                    curr = self.peek();
                    match curr {
                        '#' => {
                            self.pop();
                            tokens.push(Token::IDENT { str });
                            str = String::new();
                            let mut var_name = String::new();
                            curr = self.peek();
                            while !self.is_char_terminator(curr) {
                                self.pop();
                                var_name.push(curr);
                                curr = self.peek();
                            }
                            tokens.push(Token::VAR { name: var_name });
                        }
                        _ => {
                            str.push('$');
                        }
                    }
                }
                _ => {
                    str.push(curr);
                }
            }
            curr = self.pop();
        }
    }
}
