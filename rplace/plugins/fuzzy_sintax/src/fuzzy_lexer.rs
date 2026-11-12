use rplace::lexer::{Token, TokenResult};
use std::collections::BTreeSet;

pub struct FuzzyLexer {
    data: Vec<Token>,
    file_path: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    Def,
    For,
    Match,
    Case,
    End,
    Raw,
    EndRaw,
    Simple,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Def,
    For,
    Match,
    Case,
}

struct Command {
    kind: CommandKind,
    index: usize,
    valid: bool,
    opener: Option<BlockKind>,
    repairs: Vec<Repair>,
}

#[derive(Clone, Copy)]
enum Repair {
    AddMarker(usize),
    AddColon(usize),
}

struct OpenBlock {
    kind: BlockKind,
    command_index: usize,
}

impl FuzzyLexer {
    pub fn new(data: TokenResult) -> Self {
        Self {
            data: data.tokens,
            file_path: data.file_path,
        }
    }

    pub fn parse(self) -> TokenResult {
        let commands = find_commands(&self.data);
        let mut blocks: Vec<OpenBlock> = Vec::new();
        let mut insert_markers = BTreeSet::new();
        let mut insert_colons = BTreeSet::new();
        let mut raw_block: Option<usize> = None;

        for command_index in 0..commands.len() {
            let kind = commands[command_index].kind;
            if let Some(open_index) = raw_block {
                if kind == CommandKind::EndRaw && commands[command_index].valid {
                    apply_simple_repairs(
                        &commands[command_index],
                        &mut insert_markers,
                        &mut insert_colons,
                    );
                    apply_simple_repairs(
                        &commands[open_index],
                        &mut insert_markers,
                        &mut insert_colons,
                    );
                    raw_block = None;
                }
                continue;
            }

            match kind {
                CommandKind::Raw => {
                    if commands[command_index].valid {
                        raw_block = Some(command_index);
                    }
                }
                CommandKind::EndRaw => {}
                CommandKind::Def | CommandKind::For | CommandKind::Match => {
                    if commands[command_index].valid {
                        if let Some(opener) = commands[command_index].opener {
                            if opener != BlockKind::Match || !blocks.is_empty() {
                                blocks.push(OpenBlock {
                                    kind: opener,
                                    command_index,
                                });
                            }
                        } else {
                            apply_simple_repairs(
                                &commands[command_index],
                                &mut insert_markers,
                                &mut insert_colons,
                            );
                        }
                    }
                }
                CommandKind::Case => {
                    if commands[command_index].valid
                        && blocks
                            .last()
                            .is_some_and(|block| block.kind == BlockKind::Match)
                    {
                        blocks.push(OpenBlock {
                            kind: BlockKind::Case,
                            command_index,
                        });
                    }
                }
                CommandKind::End => {
                    if commands[command_index].valid
                        && let Some(open_block) = blocks.pop()
                    {
                        apply_simple_repairs(
                            &commands[command_index],
                            &mut insert_markers,
                            &mut insert_colons,
                        );
                        let opener = &commands[open_block.command_index];
                        apply_simple_repairs(opener, &mut insert_markers, &mut insert_colons);
                    }
                }
                CommandKind::Simple => {
                    if commands[command_index].valid {
                        apply_simple_repairs(
                            &commands[command_index],
                            &mut insert_markers,
                            &mut insert_colons,
                        );
                    }
                }
            }
        }

        let mut tokens = Vec::with_capacity(self.data.len() + 2);
        for index in 0..=self.data.len() {
            if insert_markers.contains(&index) {
                tokens.push(Token::MARK {
                    kind: "//-".to_string(),
                });
            }
            if insert_colons.contains(&index) {
                tokens.push(Token::DD);
            }
            if let Some(token) = self.data.get(index) {
                tokens.push(token.clone());
            }
        }

        TokenResult {
            tokens,
            file_path: self.file_path,
        }
    }
}

fn apply_simple_repairs(
    command: &Command,
    markers: &mut BTreeSet<usize>,
    colons: &mut BTreeSet<usize>,
) {
    for repair in &command.repairs {
        match *repair {
            Repair::AddMarker(at) => {
                markers.insert(at);
            }
            Repair::AddColon(at) => {
                colons.insert(at);
            }
        }
    }
}

fn find_commands(tokens: &[Token]) -> Vec<Command> {
    let raw_body = raw_body_mask(tokens);
    let mut in_quotes = vec![false; tokens.len()];
    let mut quote_open = false;
    for index in 0..tokens.len() {
        if raw_body[index] {
            in_quotes[index] = true;
            continue;
        }
        in_quotes[index] = quote_open;
        if matches!(tokens[index], Token::DQUOTE) && !is_escaped_quote(tokens, index) {
            quote_open = !quote_open;
        }
    }

    let mut commands = Vec::new();
    let mut line_start = 0;
    let mut index = 0;
    while index < tokens.len() {
        if matches!(tokens[index], Token::NL) {
            line_start = index + 1;
            index += 1;
            continue;
        }
        if matches!(tokens[index], Token::EOF) {
            break;
        }
        if in_quotes[index] {
            index += 1;
            continue;
        }

        if matches!(tokens[index], Token::MARK { .. }) {
            let keyword = skip_spaces(tokens, index + 1);
            if keyword < tokens.len() && !in_quotes[keyword] {
                if let Some(command) = parse_command(tokens, keyword, true) {
                    commands.push(command);
                }
            }
        } else if is_first_nonspace_on_line(tokens, line_start, index)
            && !matches!(tokens[index], Token::SPACE)
        {
            if let Some(command) = parse_command(tokens, index, false) {
                commands.push(command);
            }
        }
        index += 1;
    }
    commands.sort_by_key(|command| command.index);
    commands
}

fn raw_body_mask(tokens: &[Token]) -> Vec<bool> {
    let mut mask = vec![false; tokens.len()];
    let mut quote_open = false;
    let mut line_start = 0;

    while line_start < tokens.len() {
        let mut line_end = line_start;
        while line_end < tokens.len() && !matches!(tokens[line_end], Token::NL | Token::EOF) {
            line_end += 1;
        }
        let after_line = if matches!(tokens.get(line_end), Some(Token::NL)) {
            line_end + 1
        } else {
            line_end
        };

        if !quote_open
            && let Some((command_index, marked)) = command_at_line(tokens, line_start, line_end)
            && let Some(command) = parse_command(tokens, command_index, marked)
            && command.valid
            && command.kind == CommandKind::Raw
        {
            let body_start = after_line;
            let mut close_line_start = body_start;
            let mut closed = false;
            while close_line_start < tokens.len() {
                let mut close_line_end = close_line_start;
                while close_line_end < tokens.len()
                    && !matches!(tokens[close_line_end], Token::NL | Token::EOF)
                {
                    close_line_end += 1;
                }
                let close_after_line = if matches!(tokens.get(close_line_end), Some(Token::NL)) {
                    close_line_end + 1
                } else {
                    close_line_end
                };
                let is_close = command_at_line(tokens, close_line_start, close_line_end)
                    .and_then(|(index, marked)| parse_command(tokens, index, marked))
                    .is_some_and(|command| command.kind == CommandKind::EndRaw && command.valid);
                if is_close {
                    mask[body_start..close_line_start].fill(true);
                    line_start = close_after_line;
                    closed = true;
                    break;
                }
                if close_after_line == close_line_start {
                    break;
                }
                close_line_start = close_after_line;
            }
            if !closed {
                mask[body_start..].fill(true);
                break;
            }
            continue;
        }

        for index in line_start..after_line {
            if matches!(tokens[index], Token::DQUOTE) && !is_escaped_quote(tokens, index) {
                quote_open = !quote_open;
            }
        }
        if after_line == line_start {
            break;
        }
        line_start = after_line;
    }

    mask
}

fn command_at_line(tokens: &[Token], line_start: usize, line_end: usize) -> Option<(usize, bool)> {
    let first = skip_spaces(tokens, line_start);
    if first >= line_end {
        return None;
    }
    if matches!(tokens[first], Token::MARK { .. }) {
        let command = skip_spaces(tokens, first + 1);
        (command < line_end).then_some((command, true))
    } else {
        Some((first, false))
    }
}

fn is_escaped_quote(tokens: &[Token], index: usize) -> bool {
    let mut cursor = index;
    let mut backslashes = 0;
    while cursor > 0 && matches!(tokens[cursor - 1], Token::BSLASH) {
        cursor -= 1;
        backslashes += 1;
    }
    backslashes % 2 == 1
}

fn is_first_nonspace_on_line(tokens: &[Token], start: usize, index: usize) -> bool {
    tokens[start..index]
        .iter()
        .all(|token| matches!(token, Token::SPACE))
}

fn parse_command(tokens: &[Token], index: usize, marked: bool) -> Option<Command> {
    let end = command_end(tokens, index);
    let (kind, valid, colon, opener) = match tokens.get(index)? {
        Token::DEF => {
            let (_, colon) = parse_definition_line(tokens, index, end)?;
            let immediate = matches!(
                tokens.get(skip_spaces(tokens, index + 1)),
                Some(Token::IDENT { .. } | Token::PLACE)
            ) && matches!(
                tokens.get(skip_spaces(tokens, skip_spaces(tokens, index + 1) + 1)),
                Some(Token::PLACE | Token::DERIVE)
            );
            (
                CommandKind::Def,
                true,
                colon,
                (!immediate).then_some(BlockKind::Def),
            )
        }
        Token::FOR => {
            let (valid, colon) = parse_for_header(tokens, index, end);
            (
                CommandKind::For,
                valid,
                colon,
                valid.then_some(BlockKind::For),
            )
        }
        Token::MATCH => {
            let (valid, colon) = parse_single_ident_header(tokens, index, end);
            (
                CommandKind::Match,
                valid,
                colon,
                valid.then_some(BlockKind::Match),
            )
        }
        Token::CASE => {
            let (valid, colon) = parse_single_ident_header(tokens, index, end);
            (CommandKind::Case, valid, colon, None)
        }
        Token::END => {
            let colon = exact_colon_after(tokens, index, end);
            (
                CommandKind::End,
                colon.is_some() || only_spaces(tokens, index + 1, end),
                colon,
                None,
            )
        }
        Token::IDENT { str } if str == "raw" || str == "endraw" => {
            let raw_end = trim_raw_carriage_returns(tokens, end);
            let colon = exact_colon_after(tokens, index, raw_end);
            let kind = if matches!(&tokens[index], Token::IDENT { str } if str == "raw") {
                CommandKind::Raw
            } else {
                CommandKind::EndRaw
            };
            let valid = match colon {
                Some(colon) => only_raw_boundary_whitespace(tokens, colon + 1, raw_end),
                None => only_raw_boundary_whitespace(tokens, index + 1, raw_end),
            };
            (kind, valid, colon, None)
        }
        token if is_simple_command(token) => {
            let (valid, colon) = parse_simple_header(tokens, index, end);
            (CommandKind::Simple, valid, colon, None)
        }
        _ => return None,
    };

    if !valid {
        return None;
    }

    let repair_end = if matches!(kind, CommandKind::Raw | CommandKind::EndRaw) {
        trim_raw_carriage_returns(tokens, end)
    } else {
        end
    };
    let mut repairs = Vec::new();
    if !marked && colon.is_some() {
        repairs.push(Repair::AddMarker(index));
    }
    if colon.is_none() && marked {
        repairs.push(Repair::AddColon(repair_end));
    }
    if !marked
        && colon.is_none()
        && matches!(
            kind,
            CommandKind::Def
                | CommandKind::For
                | CommandKind::Match
                | CommandKind::Case
                | CommandKind::End
                | CommandKind::Raw
                | CommandKind::EndRaw
        )
    {
        repairs.push(Repair::AddMarker(index));
        repairs.push(Repair::AddColon(repair_end));
    }

    Some(Command {
        kind,
        index,
        valid,
        opener,
        repairs,
    })
}

fn command_end(tokens: &[Token], start: usize) -> usize {
    let mut index = start;
    let mut in_quotes = false;
    while index < tokens.len() {
        if !in_quotes && matches!(tokens[index], Token::NL | Token::EOF | Token::MARK { .. }) {
            break;
        }
        if matches!(tokens[index], Token::DQUOTE) && !is_escaped_quote(tokens, index) {
            in_quotes = !in_quotes;
        }
        index += 1;
    }
    index
}

fn trim_raw_carriage_returns(tokens: &[Token], mut end: usize) -> usize {
    while end > 0 && matches!(tokens.get(end - 1), Some(Token::IDENT { str }) if str == "\r") {
        end -= 1;
    }
    end
}

fn is_simple_command(token: &Token) -> bool {
    matches!(
        token,
        Token::PLACE
            | Token::INCLUDE
            | Token::CREATE
            | Token::DERIVE
            | Token::PARSE
            | Token::MOD
            | Token::PIPE
            | Token::VAR
    )
}

fn parse_simple_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    match tokens.get(index) {
        Some(Token::PLACE) => parse_place_header(tokens, index, end),
        Some(Token::INCLUDE | Token::PARSE | Token::MOD) => {
            let mut cursor = skip_spaces(tokens, index + 1);
            let start = cursor;
            while cursor < end && !matches!(tokens[cursor], Token::DD | Token::SPACE) {
                cursor += 1;
            }
            if cursor == start {
                return (false, None);
            }
            cursor = skip_spaces(tokens, cursor);
            let colon = (cursor < end && matches!(tokens[cursor], Token::DD)).then_some(cursor);
            (colon.is_some() || cursor == end, colon)
        }
        Some(Token::CREATE) => parse_create_header(tokens, index, end),
        Some(Token::DERIVE) => parse_derive_header(tokens, index, end),
        Some(Token::PIPE) => {
            let colon = exact_colon_after(tokens, index, end);
            (
                colon.is_some() || only_spaces(tokens, index + 1, end),
                colon,
            )
        }
        Some(Token::VAR) => parse_set_var_header(tokens, index, end),
        _ => (false, None),
    }
}

fn parse_place_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    let mut cursor = skip_spaces(tokens, index + 1);
    if !is_name(tokens.get(cursor)) {
        return (false, None);
    }
    cursor += 1;
    cursor = skip_spaces(tokens, cursor);
    if matches!(tokens.get(cursor), Some(Token::WHERE)) {
        let Some(next) = parse_assignments(tokens, cursor + 1, false) else {
            return (false, None);
        };
        cursor = next;
        cursor = skip_spaces(tokens, cursor);
    }
    let colon = (cursor < end && matches!(tokens[cursor], Token::DD)).then_some(cursor);
    (colon.is_some() || cursor == end, colon)
}

fn parse_create_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    let mut cursor = skip_spaces(tokens, index + 1);
    let path_start = cursor;
    while cursor < end && !matches!(tokens[cursor], Token::SPACE | Token::DD | Token::PLACE) {
        cursor += 1;
    }
    if cursor == path_start {
        return (false, None);
    }
    cursor = skip_spaces(tokens, cursor);
    if matches!(tokens.get(cursor), Some(Token::DD)) {
        return (true, Some(cursor));
    }
    if cursor == end {
        return (true, None);
    }
    if matches!(tokens.get(cursor), Some(Token::PLACE)) {
        return parse_place_header(tokens, cursor, end);
    }
    (false, None)
}

fn parse_derive_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    let mut cursor = skip_spaces(tokens, index + 1);
    if !matches!(tokens.get(cursor), Some(Token::WHERE)) {
        while cursor < end && !matches!(tokens[cursor], Token::SPACE | Token::WHERE | Token::DD) {
            cursor += 1;
        }
        cursor = skip_spaces(tokens, cursor);
    }
    if !matches!(tokens.get(cursor), Some(Token::WHERE)) {
        return (false, None);
    }
    let Some(cursor) = parse_assignments(tokens, cursor + 1, false) else {
        return (false, None);
    };
    let cursor = skip_spaces(tokens, cursor);
    let colon = (cursor < end && matches!(tokens[cursor], Token::DD)).then_some(cursor);
    (colon.is_some() || cursor == end, colon)
}

fn parse_set_var_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    let mut cursor = skip_spaces(tokens, index + 1);
    if !matches!(tokens.get(cursor), Some(Token::IDENT { .. })) {
        return (false, None);
    }
    cursor += 1;
    while matches!(tokens.get(cursor), Some(Token::DOT)) {
        cursor = skip_spaces(tokens, cursor + 1);
        if !matches!(tokens.get(cursor), Some(Token::IDENT { .. })) {
            return (false, None);
        }
        cursor += 1;
    }
    cursor = skip_spaces(tokens, cursor);
    if !matches!(tokens.get(cursor), Some(Token::EQUALS)) {
        return (false, None);
    }
    let Some(value_end) = skip_assignment_value(tokens, skip_spaces(tokens, cursor + 1)) else {
        return (false, None);
    };
    let value_end = skip_spaces(tokens, value_end);
    let colon = (value_end < end && matches!(tokens[value_end], Token::DD)).then_some(value_end);
    (colon.is_some() || value_end == end, colon)
}

fn parse_for_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    let mut cursor = skip_spaces(tokens, index + 1);
    loop {
        if !is_name(tokens.get(cursor)) {
            return (false, None);
        }
        cursor = skip_spaces(tokens, cursor + 1);
        match tokens.get(cursor) {
            Some(Token::COMMA) => cursor = skip_spaces(tokens, cursor + 1),
            Some(Token::IN) => {
                cursor = skip_spaces(tokens, cursor + 1);
                break;
            }
            _ => return (false, None),
        }
    }
    let range = parse_range(tokens, &mut cursor);
    if !range {
        if matches!(tokens.get(cursor), Some(Token::VAR | Token::GLOBAL)) {
            cursor += 1;
            if !matches!(tokens.get(cursor), Some(Token::IDENT { .. })) {
                return (false, None);
            }
            cursor += 1;
        } else if is_name(tokens.get(cursor)) {
            cursor += 1;
        } else {
            return (false, None);
        }
    }
    cursor = skip_spaces(tokens, cursor);
    let colon = (cursor < end && matches!(tokens[cursor], Token::DD)).then_some(cursor);
    (colon.is_some() || cursor == end, colon)
}

fn parse_range(tokens: &[Token], cursor: &mut usize) -> bool {
    let start = *cursor;
    let bound = |tokens: &[Token], cursor: &mut usize| -> bool {
        match tokens.get(*cursor) {
            Some(Token::IDENT { str }) if str.chars().all(|ch| ch.is_ascii_digit()) => {
                *cursor += 1;
                true
            }
            Some(Token::VAR | Token::GLOBAL) => {
                *cursor += 1;
                if matches!(tokens.get(*cursor), Some(Token::IDENT { .. })) {
                    *cursor += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    };
    if !bound(tokens, cursor) {
        *cursor = start;
        return false;
    }
    let after_start = *cursor;
    if !matches!(tokens.get(*cursor), Some(Token::DOT))
        || !matches!(tokens.get(*cursor + 1), Some(Token::DOT))
    {
        *cursor = start;
        return false;
    }
    *cursor += 2;
    if !bound(tokens, cursor) {
        *cursor = start;
        return false;
    }
    *cursor > after_start
}

fn parse_single_ident_header(tokens: &[Token], index: usize, end: usize) -> (bool, Option<usize>) {
    let cursor = skip_spaces(tokens, index + 1);
    if !matches!(tokens.get(cursor), Some(Token::IDENT { .. })) {
        return (false, None);
    }
    let cursor = skip_spaces(tokens, cursor + 1);
    let colon = (cursor < end && matches!(tokens[cursor], Token::DD)).then_some(cursor);
    (colon.is_some() || cursor == end, colon)
}

fn exact_colon_after(tokens: &[Token], index: usize, end: usize) -> Option<usize> {
    let cursor = skip_spaces(tokens, index + 1);
    (cursor < end && matches!(tokens[cursor], Token::DD)).then_some(cursor)
}

fn only_spaces(tokens: &[Token], start: usize, end: usize) -> bool {
    tokens[start..end]
        .iter()
        .all(|token| matches!(token, Token::SPACE))
}

fn only_raw_boundary_whitespace(tokens: &[Token], start: usize, end: usize) -> bool {
    tokens[start..end].iter().all(|token| {
        matches!(token, Token::SPACE) || matches!(token, Token::IDENT { str } if str == "\r")
    })
}

fn is_name(token: Option<&Token>) -> bool {
    matches!(token, Some(Token::IDENT { .. }))
        || token.and_then(Token::try_get_soft_keyword).is_some()
}

fn parse_definition_line(
    tokens: &[Token],
    definition_index: usize,
    end: usize,
) -> Option<(usize, Option<usize>)> {
    // Accept the declaration options the parser supports, stopping at its colon or line end.
    let mut index = skip_spaces(tokens, definition_index + 1);
    if !matches!(tokens.get(index), Some(Token::IDENT { .. } | Token::PLACE)) {
        return None;
    }
    index += 1;

    loop {
        index = skip_spaces(tokens, index);
        if index >= end {
            return Some((end, None));
        }
        match tokens.get(index)? {
            Token::DD => return Some((index, Some(index))),
            Token::NL | Token::EOF | Token::MARK { .. } => return Some((index, None)),
            Token::PLACE => {
                index = skip_spaces(tokens, index + 1);
                if !matches!(tokens.get(index), Some(Token::IDENT { .. })) {
                    return None;
                }
                index += 1;
            }
            Token::DERIVE => {
                index = skip_spaces(tokens, index + 1);
                if matches!(tokens.get(index), Some(Token::IDENT { .. })) {
                    loop {
                        match tokens.get(index) {
                            Some(Token::IDENT { .. } | Token::DOT) => index += 1,
                            Some(Token::SPACE) => {
                                index = skip_spaces(tokens, index);
                                break;
                            }
                            _ => return None,
                        }
                    }
                }
                if !matches!(tokens.get(index), Some(Token::WHERE)) {
                    return None;
                }
                index = parse_assignments(tokens, index + 1, false)?;
            }
            Token::WHEN => {
                index = parse_assignments(tokens, index + 1, true)?;
            }
            Token::WHERE => {
                index = parse_assignments(tokens, index + 1, false)?;
            }
            _ => return None,
        }
    }
}

fn parse_assignments(tokens: &[Token], mut index: usize, conditions: bool) -> Option<usize> {
    loop {
        index = skip_spaces(tokens, index);
        if !matches!(tokens.get(index), Some(Token::IDENT { .. })) {
            return None;
        }
        index = skip_spaces(tokens, index + 1);
        if !matches!(tokens.get(index), Some(Token::EQUALS)) {
            return None;
        }
        index = skip_spaces(tokens, index + 1);
        if conditions {
            if !matches!(tokens.get(index), Some(Token::IDENT { .. })) {
                return None;
            }
            index += 1;
        } else {
            index = skip_assignment_value(tokens, index)?;
        }
        index = skip_spaces(tokens, index);
        if matches!(tokens.get(index), Some(Token::COMMA)) {
            index += 1;
            continue;
        }
        return Some(index);
    }
}

fn skip_assignment_value(tokens: &[Token], mut index: usize) -> Option<usize> {
    let start = index;
    let mut nesting = 0usize;
    let mut in_quotes = false;
    // Commas and declaration keywords only delimit values outside quotes and nested collections.
    while let Some(token) = tokens.get(index) {
        if nesting == 0 && !in_quotes {
            match token {
                Token::SPACE
                | Token::COMMA
                | Token::DD
                | Token::NL
                | Token::EOF
                | Token::WHEN
                | Token::WHERE
                | Token::PLACE
                | Token::DERIVE => break,
                _ => {}
            }
        }

        match token {
            Token::DQUOTE if !is_escaped_quote(tokens, index) => in_quotes = !in_quotes,
            Token::LSRQBRACK | Token::LPAREN if !in_quotes => nesting += 1,
            Token::RSRQBRACK | Token::RPAREN if !in_quotes => {
                nesting = nesting.checked_sub(1)?;
            }
            _ => {}
        }
        index += 1;
    }

    if index == start { None } else { Some(index) }
}

fn skip_spaces(tokens: &[Token], mut index: usize) -> usize {
    while matches!(tokens.get(index), Some(Token::SPACE)) {
        index += 1;
    }
    index
}
