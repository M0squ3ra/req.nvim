use crate::req::parser::ast::{ReqBlock, ReqDocument, ReqLine};

use super::directive::parse_directive;
use super::marker::parse_request_marker;

/// Parses raw `.req` text into a document AST.
///
/// This stage only splits the input into request blocks and classifies each
/// line. It does not validate request-line, header, or body semantics.
pub fn parse_document(input: &str) -> ReqDocument {
    let mut requests = Vec::new();
    let mut current_block: Option<ReqBlock> = None;

    for (index, line) in input.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if let Some(name) = parse_request_marker(trimmed) {
            if let Some(block) = current_block.take() {
                requests.push(block);
            }

            current_block = Some(ReqBlock {
                name,
                start_line: line_number + 1,
                lines: Vec::new(),
            });

            continue;
        }

        let block = current_block.get_or_insert_with(|| ReqBlock {
            name: None,
            start_line: line_number,
            lines: Vec::new(),
        });

        block.lines.push(parse_line(line, trimmed));
    }

    if let Some(block) = current_block {
        requests.push(block);
    }

    ReqDocument { requests }
}

/// Classifies a single line inside a request block.
fn parse_line(raw: &str, trimmed: &str) -> ReqLine {
    if trimmed.is_empty() {
        return ReqLine::Empty;
    }

    if let Some(directive) = parse_directive(trimmed) {
        return ReqLine::Directive(directive);
    }

    ReqLine::Raw(raw.to_string())
}
