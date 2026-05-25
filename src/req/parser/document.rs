use crate::req::parser::ast::{ReqBlock, ReqDocument, ReqLine};

use super::directive::parse_directive;
use super::marker::parse_request_marker;

/// Parses raw `.req` text into a document AST.
///
/// This stage only splits the input into request blocks and classifies each
/// line. It does not validate request-line, header, or body semantics.
pub fn parse_document(input: &str) -> ReqDocument {
    DocumentParser::new(input).parse()
}

struct DocumentParser<'a> {
    input: &'a str,
    requests: Vec<ReqBlock>,
    current_block: Option<ReqBlock>,
    state: DocumentParseState,
    pending_prelude: Vec<(usize, ReqLine)>,
    next_prelude: Vec<(usize, ReqLine)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentParseState {
    BeforeRequest,
    InHead,
    InBody,
}

impl<'a> DocumentParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            requests: Vec::new(),
            current_block: None,
            state: DocumentParseState::BeforeRequest,
            pending_prelude: Vec::new(),
            next_prelude: Vec::new(),
        }
    }

    fn parse(mut self) -> ReqDocument {
        for (index, line) in self.input.lines().enumerate() {
            self.handle_line(index + 1, line);
        }

        self.finish_current_block();

        ReqDocument {
            requests: self.requests,
        }
    }

    fn handle_line(&mut self, line_number: usize, raw: &str) {
        let trimmed = raw.trim();

        if let Some(name) = parse_request_marker(trimmed) {
            self.start_marked_block(line_number, name);
            return;
        }

        let line = parse_line(raw, trimmed);
        let is_request_line =
            matches!(&line, ReqLine::Raw(raw) if looks_like_request_line(raw.trim()));

        if self.should_collect_initial_prelude(&line, is_request_line) {
            self.pending_prelude.push((line_number, line));
            return;
        }

        if self.should_start_next_unmarked_block(is_request_line) {
            self.start_next_unmarked_block(line_number, line);
            return;
        }

        if self.current_block.is_some() {
            self.push_to_current_block(line_number, line, is_request_line);
            return;
        }

        self.start_unmarked_block(line_number, line, is_request_line);
    }

    fn start_marked_block(&mut self, line_number: usize, name: Option<String>) {
        self.finish_current_block();
        self.reset_prelude();

        self.current_block = Some(ReqBlock {
            name,
            marker_line: Some(line_number),
            start_line: line_number + 1,
            lines: Vec::new(),
        });
        self.state = DocumentParseState::BeforeRequest;
    }

    fn should_collect_initial_prelude(&self, line: &ReqLine, is_request_line: bool) -> bool {
        self.current_block.is_none() && !is_request_line && is_prelude_line(line)
    }

    fn should_start_next_unmarked_block(&self, is_request_line: bool) -> bool {
        self.state != DocumentParseState::BeforeRequest && is_request_line
    }

    fn start_next_unmarked_block(&mut self, line_number: usize, line: ReqLine) {
        self.finish_current_block();

        let start_line = self
            .next_prelude
            .first()
            .map(|(line_number, _)| *line_number)
            .unwrap_or(line_number);
        let lines = self
            .next_prelude
            .drain(..)
            .map(|(_, line)| line)
            .chain(std::iter::once(line))
            .collect();

        self.current_block = Some(ReqBlock {
            name: None,
            marker_line: None,
            start_line,
            lines,
        });
        self.state = DocumentParseState::InHead;
    }

    fn push_to_current_block(
        &mut self,
        line_number: usize,
        line: ReqLine,
        is_request_line: bool,
    ) {
        if self.should_collect_next_prelude(&line) {
            self.next_prelude.push((line_number, line));
            return;
        }

        self.flush_next_prelude();

        if self.should_collect_block_prelude(&line) {
            self.pending_prelude.push((line_number, line));
            self.sync_current_block_with_pending_prelude(line_number);
            return;
        }

        if is_request_line {
            self.state = DocumentParseState::InHead;
            self.pending_prelude.clear();
        }

        if matches!(&line, ReqLine::Empty) && self.state == DocumentParseState::InHead {
            self.state = DocumentParseState::InBody;
        }

        if let Some(block) = self.current_block.as_mut() {
            block.lines.push(line);
        }
    }

    fn should_collect_next_prelude(&self, line: &ReqLine) -> bool {
        self.state == DocumentParseState::InBody && is_prelude_line(line)
    }

    fn should_collect_block_prelude(&self, line: &ReqLine) -> bool {
        self.state == DocumentParseState::BeforeRequest && is_prelude_line(line)
    }

    fn flush_next_prelude(&mut self) {
        if let Some(block) = self.current_block.as_mut() {
            block
                .lines
                .extend(self.next_prelude.drain(..).map(|(_, line)| line));
        }
    }

    fn sync_current_block_with_pending_prelude(&mut self, fallback_line: usize) {
        if let Some(block) = self.current_block.as_mut() {
            block.start_line = self
                .pending_prelude
                .first()
                .map(|(line_number, _)| *line_number)
                .unwrap_or(fallback_line);
            block.lines = self
                .pending_prelude
                .iter()
                .map(|(_, line)| line.clone())
                .collect();
        }
    }

    fn start_unmarked_block(&mut self, line_number: usize, line: ReqLine, is_request_line: bool) {
        let start_line = self
            .pending_prelude
            .first()
            .map(|(line_number, _)| *line_number)
            .unwrap_or(line_number);
        let lines = self
            .pending_prelude
            .drain(..)
            .map(|(_, line)| line)
            .chain(std::iter::once(line))
            .collect();

        self.current_block = Some(ReqBlock {
            name: None,
            marker_line: None,
            start_line,
            lines,
        });
        self.state = if is_request_line {
            DocumentParseState::InHead
        } else {
            DocumentParseState::BeforeRequest
        };
    }

    fn finish_current_block(&mut self) {
        if let Some(block) = self.current_block.take() {
            self.requests.push(block);
        }

        self.state = DocumentParseState::BeforeRequest;
    }

    fn reset_prelude(&mut self) {
        self.pending_prelude.clear();
        self.next_prelude.clear();
    }
}

/// Classifies a single line inside a request block.
fn parse_line(raw: &str, trimmed: &str) -> ReqLine {
    if trimmed.is_empty() {
        return ReqLine::Empty;
    }

    if let Some(directive) = parse_directive(trimmed) {
        return ReqLine::Directive(directive);
    }

    if is_comment(trimmed) {
        return ReqLine::Comment;
    }

    ReqLine::Raw(raw.to_string())
}

fn is_comment(line: &str) -> bool {
    line.starts_with("#") || line.starts_with("//")
}

fn is_prelude_line(line: &ReqLine) -> bool {
    matches!(line, ReqLine::Comment | ReqLine::Directive(_))
}

fn looks_like_request_line(line: &str) -> bool {
    let mut parts = line.split_whitespace();
    let Some(method) = parts.next() else {
        return false;
    };
    let Some(url) = parts.next() else {
        return false;
    };

    if parts.next().is_some() {
        return false;
    }

    is_http_method(method) && looks_like_url(url)
}

fn is_http_method(method: &str) -> bool {
    matches!(
        method,
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
    )
}

fn looks_like_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("{{")
        || url.starts_with("/")
}
