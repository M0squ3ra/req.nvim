use crate::req::model::{Header, HttpMethod, ParsedRequest, Request, RequestBody};

use super::ast::{Directive, ReqBlock, ReqLine};
use super::document::parse_document;

/// Error returned when `.req` input cannot be parsed into a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable error message.
    pub message: String,
    /// One-based line number where the error happened.
    pub line: usize,
    /// One-based column number where the error happened.
    pub column: usize,
}

/// Parsed `METHOD URL` request line.
struct RequestLine {
    method: HttpMethod,
    url: String,
}

/// Current section while lowering a request block into a `Request`.
enum ParseState {
    /// The parser is looking for the `METHOD URL` line.
    BeforeRequestLine,
    /// The parser is reading `Header: value` lines.
    InHeaders,
    /// The parser is reading body lines.
    InBody,
}

/// Parses the first request block from `.req` input into a parsed request.
pub fn parse_request(input: &str) -> Result<ParsedRequest, ParseError> {
    let document = parse_document(input);

    let block = document.requests.first().ok_or(ParseError {
        message: "Empty request".to_string(),
        line: 1,
        column: 1,
    })?;

    parse_block(block)
}

/// Lowers a parsed request block into the plugin request model.
fn parse_block(block: &ReqBlock) -> Result<ParsedRequest, ParseError> {
    let mut state = ParseState::BeforeRequestLine;
    let mut request_line = None;
    let mut envs = Vec::new();
    let mut headers = Vec::new();
    let mut body_lines = Vec::new();

    for (index, line) in block.lines.iter().enumerate() {
        let line_number = block.start_line + index;

        match state {
            ParseState::BeforeRequestLine => match line {
                ReqLine::Empty => {}
                ReqLine::Directive(Directive::Env(env)) => {
                    envs.push(env.clone());
                }
                ReqLine::Directive(_) => {}
                ReqLine::Raw(raw) => {
                    request_line = Some(parse_request_line(raw, line_number)?);
                    state = ParseState::InHeaders;
                }
            },
            ParseState::InHeaders => match line {
                ReqLine::Empty => {
                    state = ParseState::InBody;
                }
                ReqLine::Raw(raw) => {
                    headers.push(parse_header(raw, line_number)?);
                }
                ReqLine::Directive(Directive::Body) => {
                    state = ParseState::InBody;
                }
                ReqLine::Directive(_) => {
                    return Err(invalid_directive_position(line_number));
                }
            },
            ParseState::InBody => match line {
                ReqLine::Empty => {
                    body_lines.push(String::new());
                }
                ReqLine::Raw(raw) => {
                    body_lines.push(raw.clone());
                }
                ReqLine::Directive(_) => {
                    return Err(invalid_directive_position(line_number));
                }
            },
        }
    }

    let request_line = request_line.ok_or(ParseError {
        message: "Missing request line".to_string(),
        line: block.start_line,
        column: 1,
    })?;

    let body = if body_lines.is_empty() {
        None
    } else {
        Some(RequestBody::Raw(body_lines.join("\n")))
    };

    let request = Request {
        method: request_line.method,
        url: request_line.url,
        headers,
        body,
    };

    Ok(ParsedRequest {
        name: block.name.clone(),
        envs,
        request,
    })
}

fn invalid_directive_position(line_number: usize) -> ParseError {
    ParseError {
        message: "Directive must appear before request line or be @body".to_string(),
        line: line_number,
        column: 1,
    }
}

/// Parses a `METHOD URL` line.
fn parse_request_line(line: &str, line_number: usize) -> Result<RequestLine, ParseError> {
    let mut parts = line.split_whitespace();

    let method = parts.next().ok_or(ParseError {
        message: "Missing method".to_string(),
        line: line_number,
        column: 1,
    })?;

    let url = parts.next().ok_or(ParseError {
        message: "Missing url".to_string(),
        line: line_number,
        column: method.len() + 2,
    })?;

    if parts.next().is_some() {
        return Err(ParseError {
            message: "Invalid request line".to_string(),
            line: line_number,
            column: method.len() + url.len() + 3,
        });
    }

    let method = parse_method(method, line_number)?;

    Ok(RequestLine {
        method,
        url: url.to_string(),
    })
}

/// Parses a single `Header: value` line.
fn parse_header(line: &str, line_number: usize) -> Result<Header, ParseError> {
    let Some((name, value)) = line.split_once(":") else {
        return Err(ParseError {
            message: "Invalid header".to_string(),
            line: line_number,
            column: 1,
        });
    };

    let name = name.trim();
    let value = value.trim();

    if name.is_empty() {
        return Err(ParseError {
            message: "Missing header name".to_string(),
            line: line_number,
            column: 1,
        });
    }

    Ok(Header {
        name: name.to_string(),
        value: value.to_string(),
    })
}

/// Parses a supported HTTP method.
fn parse_method(method: &str, line_number: usize) -> Result<HttpMethod, ParseError> {
    match method {
        "GET" => Ok(HttpMethod::Get),
        "POST" => Ok(HttpMethod::Post),
        "PUT" => Ok(HttpMethod::Put),
        "PATCH" => Ok(HttpMethod::Patch),
        "DELETE" => Ok(HttpMethod::Delete),
        "HEAD" => Ok(HttpMethod::Head),
        "OPTIONS" => Ok(HttpMethod::Options),
        other => Err(ParseError {
            message: format!("Unsupported method: {other}"),
            line: line_number,
            column: 1,
        }),
    }
}
