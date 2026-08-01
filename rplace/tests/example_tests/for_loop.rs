use std::sync::Arc;

use anyhow::{Ok, Result};
use rplace::{config::config::CompilerConfig, lexer::Lexer, lua::lua_call_map::LuaCallMap, options::var_options::VarOptionsMap, parser::Parser, writer::writer::Writer};

const PATH:&str = "test.txt";
const OUTPUT_PATH:&str = "output_test.txt";

#[test]
pub fn for_loop_test_a() -> Result<()>{
    // idk why it adds a : ?
    let config = Arc::new(CompilerConfig::default());
    let lua_map = LuaCallMap::empty(config.clone());
    let var_options_map = Arc::new(VarOptionsMap::new(config.clone(), lua_map));
    let code = "//- def a://- for val in var:$#val//- end://- end://- place a where var = [(a),(b),(c)]:".to_string();
    let tok = Lexer::new(PATH, code).parse();
    let res = Parser::new(tok, PATH.to_string(), OUTPUT_PATH.to_string()).parse();
    let (mut replaced, config) = Writer::new(res, PATH.to_string(), OUTPUT_PATH.to_string(), config, var_options_map).replace();
    let str = replaced.file_data.pop().unwrap().data;
    assert_eq!(str, "abc");

    Ok(())
}