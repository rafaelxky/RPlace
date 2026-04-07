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
                    println!("Def declared {:?}", nodes.last());
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
                println!("Error: {:?} isn't valid after //-", curr);
                exit(1);
            }
        }
    }

    fn handle_def(&mut self) -> Node{
        let mut def_name : String = String::new();
        let mut body_data: Vec<Node> = Vec::new();
        // if can pop and is ident
        // we have //- def
        if !self.can_pop() {
            println!("Expected IDENT found EOF");
            exit(1);
        }
        if let Token::IDENT{str} = self.peek() {
            def_name = str.to_string();
            self.pop();
        } else {
            println!("Expected IDENT found {:?}", self.peek());
            exit(1);
        }
        if let Token::DD = self.peek() {
            self.pop();
        } else {
            println!("Expected \":\" found {:?}", self.peek());
            exit(1);
        }
        
        // we have //- def ident:
        loop {
            if matches!(self.peek(), Token::MARK){
                self.pop();
                if matches!(self.peek(), Token::ENDEF){
                    self.pop();
                    if matches!(self.peek(), Token::DD){
                        self.pop();
                        //- endef:
                        break;
                    }
                    println!("Expected \":\" after \"endef\" found {:?}", self.peek());
                    exit(1);
                }
                println!("Marks inside def blocks unimplemented");
                exit(1);
            }
            let curr = self.pop();
            match curr {
                // if its variable
                Token::VAR { name } => {
                    body_data.push(Node::VARTEMPLATE { name: name.to_string() });
                    println!("var");
                },
                Token::IDENT { str } => {
                    body_data.push(Node::DATA { data: str.to_string() });
                    println!("ident");
                },
                _ => {
                    
                }
            }
        }
        let body = Node::BODY { data: body_data };
        return Node::DEF { name: def_name, body: Box::new(body) };
    }
}