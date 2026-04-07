use std::{fs, str::Chars, thread::current};

#[derive(Debug,Clone)]
pub enum Token {
    LPAREN,
    RPAREN,
    HASH,
    IDENT{str: String},
    DEF,
    ENDEF,
    VAR{name: String},
    MARK,
    DD,
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
    fn peek_ahead(&self, i: usize) -> char{
        self.data[self.ptr + i]
    }
    fn can_pop(&self) -> bool {
        self.data.len() > self.ptr
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

    fn handle_ident(&mut self, char: char, tokens: &mut Vec<Token>){
        let mut str = String::new();
        // while not terminator, build ident
        str.push(char);
        
        while self.can_pop() && !self.is_char_terminator(self.peek()){
            let curr = self.pop();
            str.push(curr);
        }
        match str.as_str() {
            "def" => {
                tokens.push(Token::DEF);
            },
            "endef" => {
                tokens.push(Token::ENDEF);
            }
            _ => {
                tokens.push(Token::IDENT{str});
            }
        }
    }

    fn is_char_terminator(&self, char: char) -> bool{
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
            _ => false
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
                    if self.peek() == '/'{
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
                        } else {
                            tokens.push(Token::IDENT { str: "//".to_string()});
                        }
                    } else if self.peek() == '*'{
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
                        } else {
                            tokens.push(Token::IDENT { str: "/*".to_string()});
                        }
                    } 
                    else {
                        break;
                    }
                }
                '*' => {
                    if self.peek() == '/' {
                        self.pop();
                        if self.peek() == '-' {
                            self.pop();
                            tokens.push(Token::MARK);
                        } else {
                            tokens.push(Token::IDENT { str: "*/".to_string()});
                        }
                    }
                }
                ':' => {
                    tokens.push(Token::DD);
                }
                '$' => {
                    if self.peek() == '#'{
                        self.pop();
                        self.handle_var(tokens);
                    }
                }
                _ => {
                    self.handle_ident(current,tokens);
                },
            }
        }
    }
    fn handle_var(&mut self,tokens: &mut Vec<Token>){
        let mut var_name = String::new();
        while self.can_pop() && !self.is_char_terminator(self.peek()) {
            let char = self.pop();
            var_name.push(char);
        }
        tokens.push(Token::VAR{name: var_name});
    }
}
