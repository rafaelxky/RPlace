use anyhow::{Ok, Result};
use rplace::lexer::{Lexer, Token::{self, BSLASH, COMMA, DD, DOT, DQUOTE, EOF, EQUALS, LPAREN, LSRQBRACK, NL, PLUS, QD, QUESTION, RARROW, RPAREN, RSRQBRACK}};

pub const FILE_PATH: &str = "example_file.txt";

#[test]
pub fn test_ident() -> Result<()>{
    const TEXT: &str = "hello world";
    let lexer = Lexer::new(FILE_PATH, TEXT.to_string());
    let toks= lexer.parse().tokens;
    assert_eq!(toks.len(), 4);
    assert_eq!(toks[0], Token::IDENT { str: "hello".to_string() });
    assert_eq!(toks[1], Token::SPACE);
    assert_eq!(toks[2], Token::IDENT { str: "world".to_string() });
    assert_eq!(toks[3], Token::EOF);
    Ok(())
}
#[test]
pub fn test_ident_underscore() -> Result<()>{
    const TEXT: &str = "hello_world";
    let lexer = Lexer::new(FILE_PATH, TEXT.to_string());
    let toks= lexer.parse().tokens;
    assert_eq!(toks.len(), 2);
    assert_eq!(toks[0], Token::IDENT { str: "hello_world".to_string() });
    assert_eq!(toks[1], Token::EOF);
    Ok(())
}
#[test]
pub fn test_many_words() -> Result<()>{
    const EXPECTED_LEN: usize = 2;
    let def = Lexer::new(FILE_PATH, "def".to_string()).parse().tokens;
    let end = Lexer::new(FILE_PATH, "end".to_string()).parse().tokens;
    let place = Lexer::new(FILE_PATH, "place".to_string()).parse().tokens;
    let wher = Lexer::new(FILE_PATH, "where".to_string()).parse().tokens;
    let include = Lexer::new(FILE_PATH, "include".to_string()).parse().tokens;
    let when = Lexer::new(FILE_PATH, "when".to_string()).parse().tokens;
    let create = Lexer::new(FILE_PATH, "create".to_string()).parse().tokens;
    let derive = Lexer::new(FILE_PATH, "derive".to_string()).parse().tokens;
    let case = Lexer::new(FILE_PATH, "case".to_string()).parse().tokens;
    let matc = Lexer::new(FILE_PATH, "match".to_string()).parse().tokens;
    let parse = Lexer::new(FILE_PATH, "parse".to_string()).parse().tokens;
    assert_eq!(def.len(), EXPECTED_LEN);
    assert_eq!(end.len(), EXPECTED_LEN);
    assert_eq!(place.len(), EXPECTED_LEN);
    assert_eq!(wher.len(), EXPECTED_LEN);
    assert_eq!(include.len(), EXPECTED_LEN);
    assert_eq!(when.len(), EXPECTED_LEN);
    assert_eq!(create.len(), EXPECTED_LEN);
    assert_eq!(derive.len(), EXPECTED_LEN);
    assert_eq!(case.len(), EXPECTED_LEN);
    assert_eq!(matc.len(), EXPECTED_LEN);
    assert_eq!(parse.len(), EXPECTED_LEN);
    assert_eq!(def[0], Token::DEF);
    assert_eq!(end[0], Token::END);
    assert_eq!(place[0], Token::PLACE);
    assert_eq!(wher[0], Token::WHERE);
    assert_eq!(include[0], Token::INCLUDE);
    assert_eq!(when[0], Token::WHEN);
    assert_eq!(create[0], Token::CREATE);
    assert_eq!(derive[0], Token::DERIVE);
    assert_eq!(case[0], Token::CASE);
    assert_eq!(matc[0], Token::MATCH);
    assert_eq!(parse[0], Token::PARSE);
    Ok(())
}
#[test]
pub fn test_mark() -> Result<()>{ 
    const EXPECTED_LEN: usize = 2;
    let mark_a = Lexer::new(FILE_PATH, "//-".to_string()).parse().tokens;
    let mark_b = Lexer::new(FILE_PATH, "/*-".to_string()).parse().tokens;
    let mark_c = Lexer::new(FILE_PATH, "*///-".to_string()).parse().tokens;
    let mark_d = Lexer::new(FILE_PATH, "-*/".to_string()).parse().tokens;
    assert_eq!(mark_a.len(), EXPECTED_LEN);
    assert_eq!(mark_b.len(), EXPECTED_LEN);
    assert_eq!(mark_c.len(), EXPECTED_LEN);
    assert_eq!(mark_d.len(), EXPECTED_LEN);
    assert_eq!(mark_a[0], Token::MARK { kind: "//-".to_string() });
    assert_eq!(mark_b[0], Token::MARK { kind: "/*-".to_string() });
    assert_eq!(mark_c[0], Token::MARK { kind: "*///-".to_string() });
    assert_eq!(mark_d[0], Token::MARK { kind: "-*/".to_string() });
    Ok(())
}
#[test]
pub fn test_special() -> Result<()>{
    const EXPECTED_LEN: usize = 2;
    let dd = Lexer::new(FILE_PATH, ":".to_string()).parse().tokens;
    let nl = Lexer::new(FILE_PATH, "\n".to_string()).parse().tokens;
    let equals = Lexer::new(FILE_PATH, "=".to_string()).parse().tokens;
    let comma = Lexer::new(FILE_PATH, ",".to_string()).parse().tokens;
    let lsrqbrack = Lexer::new(FILE_PATH, "[".to_string()).parse().tokens;
    let rsrqbrack = Lexer::new(FILE_PATH, "]".to_string()).parse().tokens;
    let dquote = Lexer::new(FILE_PATH, "\"".to_string()).parse().tokens;
    let rarrow = Lexer::new(FILE_PATH, "->".to_string()).parse().tokens;
    let plus = Lexer::new(FILE_PATH, "+".to_string()).parse().tokens;
    let bslash = Lexer::new(FILE_PATH, "\\".to_string()).parse().tokens;
    let qd = Lexer::new(FILE_PATH, "::".to_string()).parse().tokens;
    let lparen = Lexer::new(FILE_PATH, "(".to_string()).parse().tokens;
    let rparen = Lexer::new(FILE_PATH, ")".to_string()).parse().tokens;
    let dot = Lexer::new(FILE_PATH, ".".to_string()).parse().tokens;
    let question = Lexer::new(FILE_PATH, "?".to_string()).parse().tokens;
    let eof = Lexer::new(FILE_PATH, "".to_string()).parse().tokens;
    assert_eq!(dd.len(), EXPECTED_LEN);
    assert_eq!(nl.len(), EXPECTED_LEN);
    assert_eq!(equals.len(), EXPECTED_LEN);
    assert_eq!(comma.len(), EXPECTED_LEN);
    assert_eq!(lsrqbrack.len(), EXPECTED_LEN);
    assert_eq!(rsrqbrack.len(), EXPECTED_LEN);
    assert_eq!(dquote.len(), EXPECTED_LEN);
    assert_eq!(rarrow.len(), EXPECTED_LEN);
    assert_eq!(plus.len(), EXPECTED_LEN);
    assert_eq!(bslash.len(), EXPECTED_LEN);
    assert_eq!(qd.len(), EXPECTED_LEN);
    assert_eq!(lparen.len(), EXPECTED_LEN);
    assert_eq!(rparen.len(), EXPECTED_LEN);
    assert_eq!(dot.len(), EXPECTED_LEN);
    assert_eq!(question.len(), EXPECTED_LEN);
    assert_eq!(eof.len(), EXPECTED_LEN - 1);
    assert_eq!(dd[0], DD);
    assert_eq!(nl[0], NL);
    assert_eq!(equals[0], EQUALS);
    assert_eq!(comma[0], COMMA);
    assert_eq!(lsrqbrack[0], LSRQBRACK);
    assert_eq!(rsrqbrack[0], RSRQBRACK);
    assert_eq!(dquote[0], DQUOTE);
    assert_eq!(rarrow[0], RARROW);
    assert_eq!(plus[0], PLUS);
    assert_eq!(bslash[0], BSLASH);
    assert_eq!(qd[0], QD);
    assert_eq!(lparen[0], LPAREN);
    assert_eq!(rparen[0], RPAREN);
    assert_eq!(dot[0], DOT);
    assert_eq!(question[0], QUESTION);
    assert_eq!(eof[0], EOF);
    Ok(())
}
#[test]
pub fn test_continuity() -> Result<()>{
    const EXPECTED_LEN: usize = 32;
    let text: &str = "a:b\nc=d,e[f]g\"h->i+j\\k::l(m)n.o?p"; 
    let toks = Lexer::new(FILE_PATH, text.to_string()).parse().tokens;
    assert_eq!(toks.len(), EXPECTED_LEN);
    assert_eq!(toks[0], Token::IDENT { str: "a".to_string() });
    assert_eq!(toks[1], DD);
    assert_eq!(toks[2], Token::IDENT { str: "b".to_string() });
    assert_eq!(toks[3], NL);
    assert_eq!(toks[4], Token::IDENT { str: "c".to_string() });
    assert_eq!(toks[5], EQUALS);
    assert_eq!(toks[6], Token::IDENT { str: "d".to_string() });
    assert_eq!(toks[7], COMMA);
    assert_eq!(toks[8], Token::IDENT { str: "e".to_string() });
    assert_eq!(toks[9], LSRQBRACK);
    assert_eq!(toks[10], Token::IDENT { str: "f".to_string() });
    assert_eq!(toks[11], RSRQBRACK);
    assert_eq!(toks[12], Token::IDENT { str: "g".to_string() });
    assert_eq!(toks[13], DQUOTE);
    assert_eq!(toks[14], Token::IDENT { str: "h".to_string() });
    assert_eq!(toks[15], RARROW);
    assert_eq!(toks[16], Token::IDENT { str: "i".to_string() });
    assert_eq!(toks[17], PLUS);
    assert_eq!(toks[18], Token::IDENT { str: "j".to_string() });
    assert_eq!(toks[19], BSLASH);
    assert_eq!(toks[20], Token::IDENT { str: "k".to_string() });
    assert_eq!(toks[21], QD);
    assert_eq!(toks[22], Token::IDENT { str: "l".to_string() });
    assert_eq!(toks[23], LPAREN);
    assert_eq!(toks[24], Token::IDENT { str: "m".to_string() });
    assert_eq!(toks[25], RPAREN);
    assert_eq!(toks[26], Token::IDENT { str: "n".to_string() });
    assert_eq!(toks[27], DOT);
    assert_eq!(toks[28], Token::IDENT { str: "o".to_string() });
    assert_eq!(toks[29], QUESTION);
    assert_eq!(toks[30], Token::IDENT { str: "p".to_string() });
    assert_eq!(toks[31], EOF);
    Ok(())
}