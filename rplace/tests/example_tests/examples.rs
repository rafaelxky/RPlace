use anyhow::Result;

use super::{parse, parse_with_global_values};
use rplace::{config::config::CompilerConfig, run::run_options::run_inline};
use std::collections::HashMap;

#[test]
fn global_variable_reference_is_rendered() -> Result<()> {
    let code = "//- $#var = val:\n//- def demo:Hello, $g#var!//- end://- place demo:";
    assert_eq!(parse(code)?, "\nHello, val!");
    Ok(())
}

#[test]
fn shell_global_value_overrides_source_value() -> Result<()> {
    let code = "//- $#name = default:\n//- def demo:$g#name//- end://- place demo:";
    let globals = HashMap::from([("name".to_string(), "shell".to_string())]);

    assert_eq!(parse_with_global_values(code, globals)?, "\nshell");
    Ok(())
}

#[test]
fn inline_template_renders_shell_globals() -> Result<()> {
    let output = run_inline(
        "Hello, $g#name!",
        CompilerConfig::default(),
        vec![("name".to_string(), "Rafael".to_string())],
    )?;

    assert_eq!(output, "Hello, Rafael!");
    Ok(())
}

#[test]
fn inline_template_accepts_array_shell_globals() -> Result<()> {
    let output = run_inline(
        "//- def demo://- for item in items:$#item,//- end://- end://- place demo:",
        CompilerConfig::default(),
        vec![("items".to_string(), "[a,b,c]".to_string())],
    )?;

    assert_eq!(output, "a,b,c,");
    Ok(())
}

#[test]
fn inline_template_accepts_struct_shell_globals() -> Result<()> {
    let output = run_inline(
        "//- def demo://- for first,second,third in record:$#first,$#second,$#third//- end://- end://- place demo:",
        CompilerConfig::default(),
        vec![("record".to_string(), "(a,b,c)".to_string())],
    )?;

    assert_eq!(output, "a,b,c");
    Ok(())
}

#[test]
fn inline_top_level_loop_accepts_array_shell_global() -> Result<()> {
    let output = run_inline(
        "//- for val in $g#name:\n    $#val\n//- end:",
        CompilerConfig::default(),
        vec![("name".to_string(), "[a,b,c]".to_string())],
    )?;

    assert_eq!(output, "    a\n    b\n    c\n");
    Ok(())
}

#[test]
fn inline_top_level_loop_accepts_struct_shell_global() -> Result<()> {
    let output = run_inline(
        "//- for first,second in $g#record:\n$#first:$#second\n//- end:",
        CompilerConfig::default(),
        vec![("record".to_string(), "(a,b)".to_string())],
    )?;

    assert_eq!(output, "a:b\n");
    Ok(())
}

#[test]
fn inline_top_level_range_loop_binds_each_counter() -> Result<()> {
    let output = run_inline(
        "//- for i in 0..10:\n$#i,//- end:",
        CompilerConfig::default(),
        vec![],
    )?;

    assert_eq!(output, "0,1,2,3,4,5,6,7,8,9,");
    Ok(())
}

#[test]
fn range_loop_inside_template_supports_nonzero_and_empty_ranges() -> Result<()> {
    let output = run_inline(
        "//- def demo://- for i in 3..6:$#i//- end:|//- for j in 4..4:$#j//- end://- end://- place demo:",
        CompilerConfig::default(),
        vec![],
    )?;

    assert_eq!(output, "345|");
    Ok(())
}

#[test]
fn range_loop_accepts_global_variable_bounds() -> Result<()> {
    let output = run_inline(
        "//- for i in $g#start..$g#limit:$#i,//- end:",
        CompilerConfig::default(),
        vec![
            ("start".to_string(), "2".to_string()),
            ("limit".to_string(), "5".to_string()),
        ],
    )?;

    assert_eq!(output, "2,3,4,");
    Ok(())
}

#[test]
fn range_loop_accepts_template_variable_bounds() -> Result<()> {
    let output = run_inline(
        "//- def demo://- for i in $#start..$#limit:$#i,//- end://- end://- place demo where start=3, limit=6:",
        CompilerConfig::default(),
        vec![],
    )?;

    assert_eq!(output, "3,4,5,");
    Ok(())
}

#[test]
pub fn def_test() -> Result<()> {
    let code = "//- def a:a//- end://- place a:";
    let str = parse(code)?;
    assert_eq!(str, "a");
    Ok(())
}
#[test]
pub fn def_place() -> Result<()> {
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
pub fn def_test_var_options() -> Result<()> {
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
pub fn def_test_var() -> Result<()> {
    let code = "//- def a:$#var//- end://- place a where var = val:";
    let str = parse(code)?;
    assert_eq!(str, "val");
    Ok(())
}
#[test]
pub fn def_test_var_optional() -> Result<()> {
    let code = "//- def a:$#var? value//- end://- place a:";
    let str = parse(code)?;
    assert_eq!(str, " value");
    Ok(())
}
#[test]
pub fn def_test_var_dquote() -> Result<()> {
    let code = "//- def a:$#var//- end://- place a where var = \"double quote var\":";
    let str = parse(code)?;
    assert_eq!(str, "double quote var");
    Ok(())
}
#[test]
pub fn def_test_var_multiline_var() -> Result<()> {
    let code = "//- def a:$#var//- end://- place a where var = \"
double quote var
//-\":";
    let str = parse(code)?;
    assert_eq!(str, "\ndouble quote var\n");
    Ok(())
}
#[test]
pub fn raw_text_test() -> Result<()> {
    let code = "hello world :";
    let str = parse(code)?;
    assert_eq!(str, code);
    Ok(())
}

#[test]
fn raw_block_preserves_nested_rplace_syntax() -> Result<()> {
    let code = "//- raw:\n//- def greeting:\nHello, $#name!\n//- end:\n//- endraw:";

    assert_eq!(
        parse(code)?,
        "//- def greeting:\nHello, $#name!\n//- end:\n"
    );
    Ok(())
}

#[test]
fn raw_block_output_can_be_compiled_as_a_template() -> Result<()> {
    let generated = run_inline(
        "//- raw:\n//- def greeting:\nHello, $#name!\n//- end:\n//- place greeting where name=world:\n//- endraw:",
        CompilerConfig::default(),
        vec![],
    )?;
    let rendered = run_inline(&generated, CompilerConfig::default(), vec![])?;

    assert_eq!(rendered, "Hello, world!\n");
    Ok(())
}

#[test]
fn raw_block_allows_outer_variables_around_generated_template() -> Result<()> {
    let code = "//- def outer:\nPrefix $#prefix\n//- raw:\n//- def greeting:\nHello, $#name!\n//- end:\n//- endraw:\nSuffix $#suffix\n//- end:\n//- place outer where prefix=before, suffix=after:";

    assert_eq!(
        parse(code)?,
        "Prefix before\n//- def greeting:\nHello, $#name!\n//- end:\nSuffix after\n"
    );
    Ok(())
}

#[test]
fn raw_block_preserves_crlf_content() -> Result<()> {
    let code = "//- raw:\r\n//- def greeting:\r\nHello, $#name!\r\n//- end:\r\n//- endraw:";

    assert_eq!(
        parse(code)?,
        "//- def greeting:\r\nHello, $#name!\r\n//- end:\r\n"
    );
    Ok(())
}

#[test]
fn raw_word_remains_usable_as_a_regular_value() -> Result<()> {
    let code = "//- def value:$#name//- end://- place value where name=raw:";

    assert_eq!(parse(code)?, "raw");
    Ok(())
}

#[test]
fn raw_block_requires_a_closing_delimiter() {
    let result = run_inline(
        "//- raw:\ninner template text",
        CompilerConfig::default(),
        vec![],
    );

    assert!(result.is_err());
    assert!(
        result
            .expect_err("Unterminated raw block should fail")
            .to_string()
            .contains("Unterminated raw block")
    );
}

#[test]
pub fn text_test_nl() -> Result<()> {
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
pub fn text_place_when() -> Result<()> {
    let code = "//- def a when var = val1: defa //- end: //- def a when var = val2: defb //-end: //- place a where var = val2:";
    let str = parse(code)?;
    assert_eq!(str, "defb ");
    Ok(())
}
#[test]
pub fn text_place_var_plus() -> Result<()> {
    let code = "//- def a:$#var+suffix//-end://- place a where var=val:";
    let str = parse(code)?;
    assert_eq!(str, "valsuffix");
    Ok(())
}
#[test]
pub fn arrow_var() -> Result<()> {
    let code = "//- def a: /*- $#varname -> -*/ default //- end: //- place a:";
    let str = parse(code)?;
    assert_eq!(str, "default ");
    Ok(())
}
#[test]
pub fn test_match() -> Result<()> {
    let code = "//- def a: //- match var: //- case vala:case A//- end: //- case valb:case B//- end://- end://- end://- place a where var = valb:";
    let str = parse(code)?;
    assert_eq!(str, "case B");
    Ok(())
}
