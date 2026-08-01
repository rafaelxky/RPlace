use std::sync::Arc;

use anyhow::{Ok, Result};
use rplace::{
    config::config::CompilerConfig, lexer::Lexer, lua::lua_call_map::LuaCallMap,
    options::var_options::VarOptionsMap, parser::Parser, writer::writer::Writer,
};

const PATH: &str = "test.txt";
const OUTPUT_PATH: &str = "output_test.txt";

fn parse(code: &str) -> Result<String> {
    let code = code.to_string();
    let config = Arc::new(CompilerConfig::default());
    let lua_map = LuaCallMap::empty(config.clone());
    let var_options_map = Arc::new(VarOptionsMap::new(config.clone(), lua_map));
    let tok = Lexer::new(PATH, code).parse();
    let res = Parser::new(tok, PATH.to_string(), OUTPUT_PATH.to_string()).parse();
    let (mut replaced, config) = Writer::new(
        res,
        PATH.to_string(),
        OUTPUT_PATH.to_string(),
        config,
        var_options_map,
    )
    .replace();
    let str = replaced.file_data.pop().unwrap().data;
    Ok(str)
}

#[test]
pub fn for_loop_test() -> Result<()> {
    let code ="//- def a://- for val in var:$#val//- end://- end://- place a where var = [(a),(b),(c)]:";
    let str = parse(code)?;
    assert_eq!(str, "abc");

    Ok(())
}

#[test]
pub fn def_test() -> Result<()>{
    let code = "//- def a:a//- end://- place a:";
    let str = parse(code)?;
    assert_eq!(str, "a");
    Ok(())
}
#[test]
pub fn def_test_var() -> Result<()>{
    let code = "//- def a:$#var//- end://- place a where var = val:";
    let str = parse(code)?;
    assert_eq!(str, "val");
    Ok(())
}
#[test]
pub fn text_test() -> Result<()>{
    let code = "hello world :";
    let str = parse(code)?;
    assert_eq!(str, code);
    Ok(())
}
#[test]
pub fn text_test_nl() -> Result<()>{
    let code = "
    hello 
    world 
    :
    ";
    let str = parse(code)?;
    assert_eq!(str, code);
    Ok(())
}
#[test]
pub fn text_place_when() -> Result<()>{
    let code = "//- def a when var = val1: defa //- end: //- def a when var = val2: defb //-end: //- place a where var = val2:";
    let str = parse(code)?;
    assert_eq!(str, "defb ");
    Ok(())
}
#[test]
pub fn text_place_var_plus() -> Result<()>{
    let code = "//- def a:$#var+suffix//-end://- place a where var=val:";
    let str = parse(code)?;
    assert_eq!(str, "valsuffix");
    Ok(())
}
#[test]
pub fn arrow_var() -> Result<()>{
    let code = "//- def a: /*- $#varname -> -*/ default //- end: //- place a:";
    let str = parse(code)?;
    assert_eq!(str, "default ");
    Ok(())
}