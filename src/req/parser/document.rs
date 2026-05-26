use crate::req::parser::ast::{ReqBlock, ReqDocument, ReqLine};

use super::grammar::looks_like_request_line;
use super::lexer::{Token, TokenKind, lex};
use super::span::Span;

/// Parses raw `.req` text into the parser AST.
pub fn parse(input: &str) -> ReqDocument {
    DocumentParser::new(lex(input)).parse()
}

struct DocumentParser {
    blocks: Vec<ReqBlock>,
    current: Option<BlockBuilder>,
    pending_unmarked_prefix: Vec<Token>,
}

impl DocumentParser {
    fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Self {
            blocks: Vec::new(),
            current: None,
            pending_unmarked_prefix: Vec::new(),
        };

        for token in tokens {
            parser.shift(token);
        }

        parser
    }

    fn parse(mut self) -> ReqDocument {
        self.finish_current_block();

        ReqDocument {
            requests: self.blocks,
        }
    }

    fn shift(&mut self, token: Token) {
        match &token.kind {
            TokenKind::Marker(name) => self.start_marked_block(token.span, name.clone()),
            _ if self.starts_next_unmarked_block(&token) => self.start_unmarked_block(token),
            _ if self.collects_prefix_for_future_block(&token) => {
                self.pending_unmarked_prefix.push(token);
            }
            _ => self.push_line(token),
        }
    }

    fn start_marked_block(&mut self, span: Span, name: Option<String>) {
        self.finish_current_block();
        self.pending_unmarked_prefix.clear();
        self.current = Some(BlockBuilder::marked(name, span));
    }

    fn start_unmarked_block(&mut self, request_line: Token) {
        self.finish_current_block();

        let prefix = self.pending_unmarked_prefix.drain(..);
        let mut builder = BlockBuilder::unmarked();

        for token in prefix {
            builder.push(token);
        }

        builder.push(request_line);
        self.current = Some(builder);
    }

    fn push_line(&mut self, token: Token) {
        if self.current.is_none() {
            self.current = Some(BlockBuilder::unmarked());
        }

        self.flush_pending_prefix();

        if let Some(current) = self.current.as_mut() {
            current.push(token);
        }
    }

    fn finish_current_block(&mut self) {
        if let Some(builder) = self.current.take() {
            if let Some(block) = builder.finish() {
                self.blocks.push(block);
            }
        }
    }

    fn flush_pending_prefix(&mut self) {
        let tokens = self.pending_unmarked_prefix.drain(..).collect::<Vec<_>>();

        if let Some(current) = self.current.as_mut() {
            for token in tokens {
                current.push(token);
            }
        }
    }

    fn starts_next_unmarked_block(&self, token: &Token) -> bool {
        self.current
            .as_ref()
            .is_some_and(|block| block.can_reduce_before(token))
    }

    fn collects_prefix_for_future_block(&self, token: &Token) -> bool {
        self.current
            .as_ref()
            .is_some_and(|block| block.can_collect_prefix_before_next_request(token))
    }
}

struct BlockBuilder {
    name: Option<String>,
    marker_line: Option<usize>,
    start_line: Option<usize>,
    span: Span,
    lines: Vec<ReqLine>,
    state: BlockState,
}

impl BlockBuilder {
    fn marked(name: Option<String>, marker_span: Span) -> Self {
        Self {
            name,
            marker_line: Some(marker_span.start_line),
            start_line: None,
            span: marker_span,
            lines: Vec::new(),
            state: BlockState::BeforeRequestLine,
        }
    }

    fn unmarked() -> Self {
        Self {
            name: None,
            marker_line: None,
            start_line: None,
            span: Span::between(1, 1, 1),
            lines: Vec::new(),
            state: BlockState::BeforeRequestLine,
        }
    }

    fn push(&mut self, token: Token) {
        if self.start_line.is_none() {
            self.start_line = Some(token.span.start_line);
            self.span = token.span;
        } else {
            self.span.end_line = token.span.end_line;
            self.span.end_column = token.span.end_column;
        }

        self.update_state(&token);
        self.lines.push(req_line_from_token(&token));
    }

    fn finish(self) -> Option<ReqBlock> {
        let start_line = self
            .start_line
            .or_else(|| self.marker_line.map(|line| line + 1))?;

        Some(ReqBlock {
            name: self.name,
            marker_line: self.marker_line,
            start_line,
            lines: self.lines,
            span: Span::between(
                self.marker_line.unwrap_or(self.span.start_line),
                self.span.end_line,
                self.span.end_column,
            ),
        })
    }

    fn can_reduce_before(&self, token: &Token) -> bool {
        self.state == BlockState::InBody && token_is_request_line(token)
    }

    fn can_collect_prefix_before_next_request(&self, token: &Token) -> bool {
        self.state == BlockState::InBody && token_is_prefix(token)
    }

    fn update_state(&mut self, token: &Token) {
        match self.state {
            BlockState::BeforeRequestLine if token_is_request_line(token) => {
                self.state = BlockState::InHeaders;
            }
            BlockState::InHeaders if matches!(token.kind, TokenKind::Empty) => {
                self.state = BlockState::InBody;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockState {
    BeforeRequestLine,
    InHeaders,
    InBody,
}

fn req_line_from_token(token: &Token) -> ReqLine {
    match &token.kind {
        TokenKind::Marker(_) => unreachable!("markers delimit blocks and are never block lines"),
        TokenKind::Empty => ReqLine::Empty,
        TokenKind::Comment => ReqLine::Comment(token.text.clone()),
        TokenKind::Directive(directive) => ReqLine::Directive(directive.clone()),
        TokenKind::Raw => ReqLine::Raw(token.text.clone()),
    }
}

fn token_is_prefix(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Comment | TokenKind::Directive(_))
}

fn token_is_request_line(token: &Token) -> bool {
    matches!(&token.kind, TokenKind::Raw if looks_like_request_line(token.text.trim()))
}
