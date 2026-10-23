use anyhow::Result;

use super::parse;

#[test]
fn for_loop_test() -> Result<()> {
    let code =
        "//- def a://- for val in var:$#val//- end://- end://- place a where var = [(a),(b),(c)]:";
    let str = parse(code)?;
    assert_eq!(str, "abc");
    Ok(())
}

#[test]
fn for_loop_binds_multiple_values() -> Result<()> {
    let code = "//- def a://- for first,second in pairs:$#first,$#second;//- end://- end://- place a where pairs = [(one,two),(three,four)]:";
    let str = parse(code)?;
    assert_eq!(str, "one,two;three,four;");
    Ok(())
}

#[test]
fn array_can_mix_values_and_structs() -> Result<()> {
    let code = "//- def a://- for val in values:$#val//- end://- end://- place a where values = [a,(b,c)]:";
    let str = parse(code)?;
    assert_eq!(str, "ab");
    Ok(())
}

#[test]
fn array_can_contain_a_single_value() -> Result<()> {
    let code =
        "//- def a://- for val in values:$#val//- end://- end://- place a where values = [val]:";
    let str = parse(code)?;
    assert_eq!(str, "val");
    Ok(())
}

#[test]
fn array_can_contain_a_single_value_struct() -> Result<()> {
    let code =
        "//- def a://- for val in values:$#val//- end://- end://- place a where values = [(val)]:";
    let str = parse(code)?;
    assert_eq!(str, "val");
    Ok(())
}

#[test]
fn array_can_contain_named_structs() -> Result<()> {
    let code = "//- def a://- for first,second in values:$#first,$#second;//- end://- end://- place a where values = [(first=one,second=two)]:";
    let str = parse(code)?;
    assert_eq!(str, "one,two;");
    Ok(())
}

#[test]
fn empty_array_runs_no_iterations() -> Result<()> {
    let code =
        "//- def a://- for val in values:$#val//- end://- end://- place a where values = []:";
    let str = parse(code)?;
    assert_eq!(str, "");
    Ok(())
}
