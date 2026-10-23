use crate::{parser::Parser, structs::{Node, ParsingResult}};

use anyhow::{Ok, Result, anyhow};

impl Parser {
    pub fn handle_pipe(&mut self, nodes: &mut ParsingResult) -> Result<()>{
        self.remove_spaces();
        let _ = self.pop().expect_dd_or_else(||{
            anyhow!("Expected : in pipe!")
        })?;
        nodes.push(Node::PIPE);
        Ok(())
    }
}
