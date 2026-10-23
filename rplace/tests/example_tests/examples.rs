use anyhow::Result;

use super::{parse, parse_with_global_values};
use std::collections::HashMap;

#[test]
fn global_variable_reference_is_rendered() -> Result<()> {
    let code = "//- $#var = val:\n$g#var";
    assert_eq!(parse(code)?, "\nval");
    Ok(())
}

#[test]
fn shell_global_value_overrides_source_value() -> Result<()> {
    let code = "//- $#name = default:\n$g#name";
    let globals = HashMap::from([("name".to_string(), "shell".to_string())]);

    assert_eq!(parse_with_global_values(code, globals)?, "\nshell");
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
pub fn def_place() -> Result<()>{
    let code = "//- def a:$#var//- end://- def b place a where var = val://- place b:";
    let str = parse(code)?;
    assert_eq!(str, "val");
    Ok(())
}
#[test]
pub fn def_place_override() -> Result<()> {
    let code = "//- def a:$#var//- end://- def b place a where var = default://- place b where var = override:";
    let str = parse(code)?;
    assert_eq!(str, "override");
    Ok(())
}
#[test]
pub fn def_when_uses_default_overload() -> Result<()> {
    let code = "//- def a when kind = special:special//- end://- def a:default//- end://- place a:";
    let str = parse(code)?;
    assert_eq!(str, "default");
    Ok(())
}
#[test]
pub fn def_test_var_options() -> Result<()>{
    let code = "//- def a:$#var\\screaming//- end://- place a where var = val:";
    let str = parse(code)?;
    assert_eq!(str, "VAL");
    Ok(())
}
#[test]
pub fn def_test_var_case_options() -> Result<()> {
    let cases = [
        ("snakecase", "helloWorld", "hello_world"),
        ("camelcase", "hello_world", "helloWorld"),
        ("pascalcase", "hello_world", "HelloWorld"),
        ("screaming", "helloWorld", "HELLO_WORLD"),
    ];

    for (option, input, expected) in cases {
        let code = format!(
            "//- def a:$#var\\{}//- end://- place a where var = {}:",
            option, input
        );
        assert_eq!(parse(&code)?, expected, "option {option}");
    }

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
pub fn def_test_var_optional() -> Result<()>{
    let code = "//- def a:$#var? value//- end://- place a:";
    let str = parse(code)?;
    assert_eq!(str, " value");
    Ok(())
}
#[test]
pub fn def_test_var_dquote() -> Result<()>{
    let code = "//- def a:$#var//- end://- place a where var = \"double quote var\":";
    let str = parse(code)?;
    assert_eq!(str, "double quote var");
    Ok(())
}
#[test]
pub fn def_test_var_multiline_var() -> Result<()>{
    let code = 
"//- def a:$#var//- end://- place a where var = \"
double quote var
//-\":";
    let str = parse(code)?;
    assert_eq!(str, "\ndouble quote var\n");
    Ok(())
}
#[test]
pub fn raw_text_test() -> Result<()>{
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
#[test]
pub fn test_match() -> Result<()>{
    let code = 
    "//- def a: //- match var: //- case vala:case A//- end: //- case valb:case B//- end://- end://- end://- place a where var = valb:";
    let str = parse(code)?;
    assert_eq!(str, "case B");
    Ok(())
}