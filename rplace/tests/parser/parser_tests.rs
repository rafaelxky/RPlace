use anyhow::Result;
use rplace::{lexer::Lexer, parser::Parser};

fn parse_invalid(code: &str) -> Result<String> {
    let tokens = Lexer::new("invalid.txt", code.to_string()).parse();
    let result = Parser::new(
        tokens,
        "invalid.txt".to_string(),
        "output.txt".to_string(),
    )
    .parse();

    match result {
        Ok(_) => panic!("expected parser to reject {code:?}"),
        Err(error) => Ok(error.to_string()),
    }
}

#[test]
fn rejects_unknown_command() -> Result<()> {
    let error = parse_invalid("//- unknown:")?;
    assert!(error.contains("Invalid token after mark"));
    Ok(())
}

#[test]
fn rejects_definition_without_name() -> Result<()> {
    let error = parse_invalid("//- def:")?;
    assert!(error.contains("Invalid token found in def declaration name"));
    Ok(())
}

#[test]
fn rejects_invalid_when_condition() -> Result<()> {
    let error = parse_invalid("//- def a when = value:body//- end:")?;
    assert!(error.contains("Invalid token in variable name before comparison"));
    Ok(())
}

#[test]
fn rejects_unterminated_quoted_value() -> Result<()> {
    let error = parse_invalid("//- def a:$#value//- end://- place a where value=\"unterminated:")?;
    assert!(error.contains("Found EOF inside of quotation variable"));
    Ok(())
}