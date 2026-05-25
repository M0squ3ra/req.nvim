use crate::req::model::{HttpMethod, Request};

use super::ast::{ReqBlock, ReqLine};
use super::document::parse_document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub fn parse_request(input: &str) -> Result<Request, ParseError> {
    let document = parse_document(input);

    let block = document.requests.first().ok_or(ParseError {
        message: "Empty request".to_string(),
        line: 1,
        column: 1,
    })?;

    parse_block(block)
}

fn parse_block(block: &ReqBlock) -> Result<Request, ParseError> {
    for (index, line) in block.lines.iter().enumerate() {
        let ReqLine::Raw(raw) = line else {
            continue;
        };

        return parse_request_line(block.name.clone(), raw, block.start_line + index);
    }

    Err(ParseError {
        message: "Missing request line".to_string(),
        line: block.start_line,
        column: 1,
    })
}

fn parse_request_line(
    name: Option<String>,
    line: &str,
    line_number: usize,
) -> Result<Request, ParseError> {
    let mut parts = line.split_whitespace();

    let method = parts.next().ok_or(ParseError {
        message: "Missing method".to_string(),
        line: line_number,
        column: 1,
    })?;

    let url = parts.next().ok_or(ParseError {
        message: "Missing url".to_string(),
        line: line_number,
        column: 2,
    })?;

    if parts.next().is_some() {
        return Err(ParseError {
            message: "Invalid request line".to_string(),
            line: line_number,
            column: method.len() + url.len() + 3,
        });
    }

    let method = parse_method(method, line_number)?;

    Ok(Request {
        name,
        method,
        url: url.to_string(),
        headers: vec![],
        body: None,
    })
}

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
