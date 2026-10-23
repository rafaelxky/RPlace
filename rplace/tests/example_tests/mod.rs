use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Ok, Result};
use rplace::{
    config::config::CompilerConfig, lexer::Lexer, lua::lua_call_map::LuaCallMap,
    options::var_options::VarOptionsMap, parser::Parser, writer::writer::Writer,
};

pub mod arrays;
pub mod examples;

fn parse(code: &str) -> Result<String> {
    parse_with_global_values(code, HashMap::new())
}

fn parse_with_global_values(code: &str, global_values: HashMap<String, String>) -> Result<String> {
    const PATH: &str = "test.txt";
    const OUTPUT_PATH: &str = "output_test.txt";

    let code = code.to_string();
    let config = Arc::new(CompilerConfig::default());
    let lua_map = LuaCallMap::empty(config.clone());
    let var_options_map = Arc::new(VarOptionsMap::new(config.clone(), lua_map));
    let tok = Lexer::new(PATH, code).parse();
    let res = Parser::new(tok, PATH.to_string(), OUTPUT_PATH.to_string()).parse()?;
    let (mut replaced, _config) = Writer::new(
        res,
        PATH.to_string(),
        OUTPUT_PATH.to_string(),
        config,
        var_options_map,
    )
    .with_global_variables(global_values)
    .replace()?;
    let str = replaced.file_data.pop().unwrap().data;
    Ok(str)
}
