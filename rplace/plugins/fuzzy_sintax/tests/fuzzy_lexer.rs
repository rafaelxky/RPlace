use rplace::{
    lexer::{Lexer, Token, TokenResult},
    parser::Parser,
};
use rplace_fuzzy_sintax::fuzzy_lexer::FuzzyLexer;

fn fuzzy_tokens(source: &str) -> TokenResult {
    FuzzyLexer::new(Lexer::new("test", source.to_string()).parse()).parse()
}

fn raw_body(tokens: &[Token]) -> Vec<Token> {
    let raw_index = tokens
        .iter()
        .position(|token| matches!(token, Token::IDENT { str } if str == "raw"))
        .expect("Raw opener should be present");
    let body_start = tokens[raw_index + 1..]
        .iter()
        .position(|token| matches!(token, Token::NL))
        .map(|index| raw_index + index + 2)
        .expect("Raw opener should end with a newline");
    let endraw_index = tokens[body_start..]
        .iter()
        .position(|token| matches!(token, Token::IDENT { str } if str == "endraw"))
        .map(|index| body_start + index)
        .expect("Raw closer should be present");
    let body_end = tokens[..endraw_index]
        .iter()
        .rposition(|token| matches!(token, Token::NL))
        .map_or(body_start, |index| index + 1);
    tokens[body_start..body_end].to_vec()
}

#[test]
fn adds_colon_to_explicit_definition() {
    let result = fuzzy_tokens("//- def example\n//- end:");
    assert!(result.tokens.windows(2).any(|window| {
        matches!(window[0], Token::IDENT { ref str } if str == "example")
            && matches!(window[1], Token::DD)
    }));
}

#[test]
fn repairs_raw_delimiters_without_changing_the_literal_body() {
    let source = "raw\r\n//- def generated\r\n\"an unmatched quote\r\nend\r\nendraw\r\n";
    let original = Lexer::new("test", source.to_string()).parse();
    let result = FuzzyLexer::new(original.clone()).parse();
    let raw_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::IDENT { str } if str == "raw"))
        .expect("Raw opener should remain present");
    let endraw_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::IDENT { str } if str == "endraw"))
        .expect("Raw closer should remain present");

    assert!(matches!(
        result.tokens.get(raw_index - 1),
        Some(Token::MARK { .. })
    ));
    assert!(matches!(result.tokens.get(raw_index + 1), Some(Token::DD)));
    assert!(matches!(
        result.tokens.get(endraw_index - 1),
        Some(Token::MARK { .. })
    ));
    assert!(matches!(
        result.tokens.get(endraw_index + 1),
        Some(Token::DD)
    ));
    assert_eq!(raw_body(&result.tokens), raw_body(&original.tokens));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );
}

#[test]
fn repairs_unmarked_definition_when_end_count_supports_it() {
    let result = fuzzy_tokens("def example\nbody\n//- end");
    assert!(matches!(
        result.tokens.first(),
        Some(Token::MARK { kind }) if kind == "//-"
    ));
    assert!(result.tokens.windows(2).any(|window| {
        matches!(window[0], Token::IDENT { ref str } if str == "example")
            && matches!(window[1], Token::DD)
    }));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );
}

#[test]
fn parses_the_repository_fuzzy_example() {
    let result = fuzzy_tokens(include_str!("../../../ex2.txt"));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );
}

#[test]
fn adds_marker_to_unmarked_definition_with_a_matching_end() {
    let result = fuzzy_tokens("def example:\nbody\nend:");
    assert!(matches!(
        result.tokens.first(),
        Some(Token::MARK { kind }) if kind == "//-"
    ));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );
}

#[test]
fn leaves_unclosed_unmarked_definition_unchanged() {
    let original = Lexer::new("test", "def example:".to_string()).parse();
    assert_eq!(fuzzy_tokens("def example:").tokens, original.tokens);
}

#[test]
fn recognizes_quoted_definition_defaults() {
    let result = fuzzy_tokens("//- def example where value=\"a,b\":");
    assert!(result.tokens.windows(2).any(|window| {
        matches!(window[0], Token::IDENT { ref str } if str == "value")
            && matches!(window[1], Token::EQUALS)
    }));
}

#[test]
fn leaves_unsupported_plain_text_unchanged() {
    let original = Lexer::new("test", "def example".to_string()).parse();
    let result = fuzzy_tokens("def example");
    assert_eq!(result.tokens.len(), original.tokens.len());
    assert_eq!(result.file_path, original.file_path);
}

#[test]
fn preserves_existing_markers_and_file_path() {
    let result =
        FuzzyLexer::new(Lexer::new("sample.rp", "//- def example:".to_string()).parse()).parse();
    assert_eq!(result.file_path, "sample.rp");
    assert_eq!(
        result
            .tokens
            .iter()
            .filter(|token| matches!(token, Token::MARK { kind } if kind == "//-"))
            .count(),
        1
    );
}

#[test]
fn repairs_probable_end_missing_marker_and_colon() {
    let result = fuzzy_tokens("//- def example:\nbody\nend");
    let end_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::END))
        .expect("End token should remain present");

    assert!(matches!(
        result.tokens.get(end_index - 1),
        Some(Token::MARK { kind }) if kind == "//-"
    ));
    assert!(matches!(result.tokens.get(end_index + 1), Some(Token::DD)));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );
}

#[test]
fn adds_colon_to_marked_end_command() {
    let result = fuzzy_tokens("//- def example:\nbody\n//- end");
    let end_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::END))
        .expect("End token should remain present");

    assert!(matches!(result.tokens.get(end_index + 1), Some(Token::DD)));
}

#[test]
fn ignores_end_word_inside_quoted_text() {
    let result = fuzzy_tokens("//- def example:\nThe word \"end\" is quoted.\n//- end:");
    assert_eq!(
        result
            .tokens
            .iter()
            .filter(|token| matches!(token, Token::MARK { kind } if kind == "//-"))
            .count(),
        2
    );
}

#[test]
fn leaves_unpaired_end_word_in_prose_unchanged() {
    let original = Lexer::new("test", "in the end they died".to_string()).parse();
    let result = fuzzy_tokens("in the end they died");

    assert_eq!(result.tokens, original.tokens);
}

#[test]
fn leaves_end_word_in_prose_unchanged_inside_a_block() {
    let result = fuzzy_tokens("//- def example:\nin the end they died\n//- end:");
    let end_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::END))
        .expect("Prose should still lex as an END token");

    assert!(matches!(
        result.tokens.get(end_index + 1),
        Some(Token::SPACE)
    ));
    assert!(!matches!(
        result.tokens.get(end_index - 1),
        Some(Token::MARK { .. })
    ));
}

#[test]
fn leaves_unpaired_end_syntax_ambiguous() {
    let for_end_with_colon = Lexer::new("test", "end:".to_string()).parse();
    let result_with_colon = fuzzy_tokens("end:");
    assert_eq!(result_with_colon.tokens, for_end_with_colon.tokens);

    let for_plain_end = Lexer::new("test", "end".to_string()).parse();
    let result_for_plain_end = fuzzy_tokens("end");
    assert_eq!(result_for_plain_end.tokens, for_plain_end.tokens);
}

#[test]
fn adds_markers_to_complete_simple_commands() {
    let result = fuzzy_tokens("place sample:\nparse file.txt:\nmod module.txt:\nhere:");
    for command in [Token::PLACE, Token::PARSE, Token::MOD, Token::PIPE] {
        assert!(result.tokens.iter().enumerate().any(|(index, token)| {
            matches!(token, current if std::mem::discriminant(current) == std::mem::discriminant(&command))
                && matches!(result.tokens.get(index - 1), Some(Token::MARK { .. }))
        }));
    }
}

#[test]
fn repairs_missing_colons_on_marked_simple_commands() {
    let result = fuzzy_tokens(
        "//- place sample\n//- include file.txt\n//- parse file.txt\n//- mod module.txt\n//- here\n//- derive where value=thing\n//- create file.txt",
    );
    for token in [
        Token::PLACE,
        Token::INCLUDE,
        Token::PARSE,
        Token::MOD,
        Token::PIPE,
        Token::DERIVE,
        Token::CREATE,
    ] {
        let command_index = result
            .tokens
            .iter()
            .position(|current| std::mem::discriminant(current) == std::mem::discriminant(&token))
            .expect("Command token should remain present");
        assert!(
            result.tokens[command_index..]
                .iter()
                .take_while(|current| !matches!(current, Token::NL | Token::EOF))
                .any(|current| matches!(current, Token::DD))
        );
    }
}

#[test]
fn leaves_command_words_in_quoted_strings_unchanged() {
    let source = "\"def example: end:\"";
    let original = Lexer::new("test", source.to_string()).parse();
    assert_eq!(fuzzy_tokens(source).tokens, original.tokens);
}

#[test]
fn ignores_markers_inside_escaped_quoted_text() {
    let source = "//- def example:\n\"escaped \\\" //- end:\"\n//- end:";
    let original = Lexer::new("test", source.to_string()).parse();
    let result = fuzzy_tokens(source);
    assert_eq!(result.tokens, original.tokens);
}

#[test]
fn ignores_command_markers_inside_quoted_assignment_values() {
    let source = "//- place sample where value=\"quoted //- end:\":";
    let original = Lexer::new("test", source.to_string()).parse();
    assert_eq!(fuzzy_tokens(source).tokens, original.tokens);
}

#[test]
fn repairs_are_idempotent() {
    let first = fuzzy_tokens("def example\nbody\nend");
    let second = FuzzyLexer::new(TokenResult {
        tokens: first.tokens.clone(),
        file_path: first.file_path.clone(),
    })
    .parse();
    assert_eq!(second.tokens, first.tokens);
}

#[test]
fn repairs_nested_for_blocks_with_a_typed_stack() {
    let result = fuzzy_tokens(
        "//- def example:\nfor item in items:\n//- for child in item:\ntext\nend:\nend:\nend:",
    );
    let for_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::FOR))
        .expect("Loop token should remain present");
    assert!(matches!(
        result.tokens.get(for_index - 1),
        Some(Token::MARK { kind }) if kind == "//-"
    ));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );
}

#[test]
fn repairs_match_and_case_blocks_only_in_a_body_context() {
    let result = fuzzy_tokens("//- def example:\nmatch value:\ncase yes:\nhello\nend:\nend:\nend:");
    let match_index = result
        .tokens
        .iter()
        .position(|token| matches!(token, Token::MATCH))
        .expect("Match token should remain present");
    assert!(matches!(
        result.tokens.get(match_index - 1),
        Some(Token::MARK { kind }) if kind == "//-"
    ));
    assert!(
        Parser::new(result, ".".to_string(), ".".to_string())
            .parse()
            .is_ok()
    );

    let top_level_match = Lexer::new("test", "//- match value:\n//- end:".to_string()).parse();
    assert_eq!(
        FuzzyLexer::new(top_level_match.clone()).parse().tokens,
        top_level_match.tokens
    );
}

#[test]
fn repairs_missing_for_colon_only_for_a_complete_header() {
    let result = fuzzy_tokens("//- def example:\n//- for item in items\n//- end:\n//- end:");
    assert!(result.tokens.windows(2).any(|window| {
        matches!(&window[0], Token::IDENT { str } if str == "items")
            && matches!(window[1], Token::DD)
    }));
}

#[test]
fn does_not_repair_incomplete_or_trailing_garbage_headers() {
    let source = "//- def example:\n//- for item in items extra\n//- match value extra\n//- place sample where value=one two\n//- end:";
    let original = Lexer::new("test", source.to_string()).parse();
    let result = fuzzy_tokens(source);
    assert_eq!(result.tokens, original.tokens);
}
