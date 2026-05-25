use crate::req::model::{HttpMethod, Request};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub fn parse_request(input: &str) -> Result<Request, ParseError> {
    let (line_number, line) = first_non_empty_line(input)?;
    parse_request_line(line, line_number)
}

fn parse_request_line(line: &str, line_number: usize) -> Result<Request, ParseError> {
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
        name: None,
        method,
        url: url.to_string(),
        headers: vec![],
        body: None,
    })
}

fn first_non_empty_line(input: &str) -> Result<(usize, &str), ParseError> {
    for (index, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Ok((index + 1, trimmed));
        }
    }

    Err(ParseError {
        message: "Empty request".to_string(),
        line: 1, //TODO, pass first line as argument
        column: 1,
    })
}

fn parse_method(method: &str, line_number: usize) -> Result<HttpMethod, ParseError> {
    match method {
        "GET" => Ok(HttpMethod::Get),
        other => Err(ParseError {
            message: format!("Unsupported method: {other}"),
            line: line_number,
            column: 1,
        }),
    }
}
