use super::ast::Directive;
use super::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    Marker(Option<String>),
    Empty,
    Comment,
    Directive(Directive),
    Raw,
}

pub(super) fn lex(input: &str) -> Vec<Token> {
    input
        .lines()
        .enumerate()
        .map(|(index, raw)| lex_line(index + 1, raw))
        .collect()
}

fn lex_line(line_number: usize, raw: &str) -> Token {
    let trimmed = raw.trim();
    let kind = if let Some(name) = parse_request_marker(trimmed) {
        TokenKind::Marker(name)
    } else if trimmed.is_empty() {
        TokenKind::Empty
    } else if let Some(directive) = parse_directive(trimmed) {
        TokenKind::Directive(directive)
    } else if is_comment(trimmed) {
        TokenKind::Comment
    } else {
        TokenKind::Raw
    };

    Token {
        kind,
        text: raw.to_string(),
        span: Span::line(line_number, raw.len()),
    }
}

fn is_comment(line: &str) -> bool {
    line.starts_with('#') || line.starts_with("//")
}

fn parse_request_marker(line: &str) -> Option<Option<String>> {
    let name = line.strip_prefix("###")?.trim();

    if name.is_empty() {
        return Some(None);
    }

    Some(Some(name.to_string()))
}

fn parse_directive(line: &str) -> Option<Directive> {
    parse_metadata_comment(line).or_else(|| parse_inline_variable(line))
}

fn parse_metadata_comment(line: &str) -> Option<Directive> {
    let metadata = line.strip_prefix('#')?.trim();
    let mut parts = metadata.split_whitespace();
    let name = parts.next()?;

    match name {
        "@env" => parse_env(parts),
        _ => None,
    }
}

fn parse_inline_variable(line: &str) -> Option<Directive> {
    let directive = line.strip_prefix('@')?;
    parse_variable(directive)
}

fn parse_env<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Directive> {
    let env = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    Some(Directive::Env(env.to_string()))
}

fn parse_variable(variable: &str) -> Option<Directive> {
    let (name, value) = variable.split_once('=')?;
    let name = name.trim();
    let value = value.trim();

    if name.is_empty() || value.is_empty() {
        return None;
    }

    Some(Directive::Variable {
        name: name.to_string(),
        value: value.to_string(),
    })
}
