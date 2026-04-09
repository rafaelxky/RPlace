use core::panic;
use std::process::exit;

use crate::lexer::Token;

#[derive(Debug,Clone)]
pub enum Node {
    // - def template_1
    DEF{name: String, body: Box<Node>},
    // either data or var ($a)
    BODY{data: Vec<Node>},
    // def body
    DATA{data: String},
    // def variables
    VARTEMPLATE{name: String},
}
pub struct Parser {
    tokens: Vec<Token>,
    ptr: usize
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self{ 
        Self { tokens, ptr: 0 }
    }
    fn peek(&self) -> &Token{
        &self.tokens[self.ptr]
    }
    fn pop(&mut self) -> &Token{
        self.ptr = self.ptr +1;
        &self.tokens[self.ptr - 1]
    }
    fn can_pop(&self) -> bool{
        self.tokens.len() > self.ptr
    }
    pub fn parse(mut self) -> Vec<Node>{
        let mut nodes: Vec<Node> = Vec::new();
        while self.can_pop() {
            let curr = self.pop();
            match curr {
                // starts with //-
                Token::MARK => {
                    nodes.push(self.handle_line());
                }
                _ => (),
            }
        }
        nodes
    }
    fn handle_line(&mut self) -> Node{
        // we have //-
        // possible
        // def
        // endef
        let curr = self.pop();
        match curr {
            Token::DEF => {
                return self.handle_def();
            }
            _ => {
                panic!("Error: {:?} isn't valid after //-", curr);
            }
        }
    }

    fn handle_def(&mut self) -> Node{
        let mut def_name : String = String::new();
        let mut body_data: Vec<Node> = Vec::new();
        // if can pop and is ident
        // we have //- def
        if !self.can_pop() {
            panic!("Expected IDENT found EOF");
        }
        if let Token::IDENT{str} = self.peek() {
            def_name = str.to_string();
            self.pop();
        } else {
            panic!("Expected IDENT found {:?}", self.peek());
        }
        if let Token::DD = self.peek() {
            self.pop();
        } else {
            panic!("Expected \":\" found {:?}", self.peek());
        }
        
        // we have //- def ident:
        loop {
            if !self.can_pop(){
                panic!("found EOF, expected matching MARK");
            }
            if matches!(self.peek(), Token::MARK){
                self.pop();
                if !self.can_pop() {
                    panic!("expected ENDEF found EOF")
                }
                if matches!(self.peek(), Token::ENDEF){
                    self.pop();
                    if !self.can_pop() {
                        panic!("expected DD found EOF")
                    }
                    if matches!(self.peek(), Token::DD){
                        self.pop();
                        //- endef:
                        break;
                    }
                    panic!("Expected \":\" after \"endef\" found {:?}", self.peek());
                }
                panic!("Marks inside def blocks unimplemented");
            }
            let curr = self.pop();
            match curr {
                // if its variable
                Token::VAR { name } => {
                    body_data.push(Node::VARTEMPLATE { name: name.to_string() });
                },
                Token::IDENT { str } => {
                    body_data.push(Node::DATA { data: str.to_string() });
                },
                Token::DD => {
                    body_data.push(Node::DATA { data: ":".to_string() });
                },  
                Token::LPAREN => {
                    body_data.push(Node::DATA { data: "(".to_string() });
                },
                Token::RPAREN => {
                    body_data.push(Node::DATA { data: ")".to_string() });
                }
                _ => {
                    
                }
            }
        }
        let body = Node::BODY { data: body_data };
        return Node::DEF { name: def_name, body: Box::new(body) };
    }
}