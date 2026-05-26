use super::error::ParseError;
use super::model::{Header, HttpMethod, ParsedRequest, Request, RequestBody, Variable};
use super::parser::ast::{Directive, ReqBlock, ReqDocument, ReqLine};

struct RequestLine {
    method: HttpMethod,
    url: String,
}

enum LoweringState {
    BeforeRequestLine,
    InHeaders,
    InBody,
}

/// Converts the first request block in a parsed document into the executable model.
pub fn lower_first_request(document: &ReqDocument) -> Result<ParsedRequest, ParseError> {
    let block = document.requests.first().ok_or(ParseError {
        message: "Empty request".to_string(),
        line: 1,
        column: 1,
    })?;

    lower_request(block)
}

/// Converts a parser AST request block into the executable request model.
pub fn lower_request(block: &ReqBlock) -> Result<ParsedRequest, ParseError> {
    let mut state = LoweringState::BeforeRequestLine;
    let mut request_line = None;
    let mut envs = Vec::new();
    let mut vars = Vec::new();
    let mut headers = Vec::new();
    let mut body_lines = Vec::new();

    for (index, line) in block.lines.iter().enumerate() {
        let line_number = block.start_line + index;

        match state {
            LoweringState::BeforeRequestLine => match line {
                ReqLine::Empty | ReqLine::Comment(_) => {}
                ReqLine::Directive(Directive::Env(env)) => {
                    envs.push(env.clone());
                }
                ReqLine::Directive(Directive::Variable { name, value }) => {
                    vars.push(Variable {
                        name: name.clone(),
                        value: value.clone(),
                    });
                }
                ReqLine::Raw(raw) => {
                    request_line = Some(parse_request_line(raw, line_number)?);
                    state = LoweringState::InHeaders;
                }
            },
            LoweringState::InHeaders => match line {
                ReqLine::Comment(_) => {}
                ReqLine::Empty => {
                    state = LoweringState::InBody;
                }
                ReqLine::Raw(raw) => {
                    headers.push(parse_header(raw, line_number)?);
                }
                ReqLine::Directive(_) => {
                    return Err(invalid_directive_position(line_number));
                }
            },
            LoweringState::InBody => match line {
                ReqLine::Comment(raw) => {
                    body_lines.push(raw.clone());
                }
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

    Ok(ParsedRequest {
        name: block.name.clone(),
        envs,
        vars,
        request: Request {
            method: request_line.method,
            url: request_line.url,
            headers,
            body,
        },
    })
}

fn invalid_directive_position(line_number: usize) -> ParseError {
    ParseError {
        message: "Directive must appear before request line".to_string(),
        line: line_number,
        column: 1,
    }
}

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

    Ok(RequestLine {
        method: parse_method(method, line_number)?,
        url: url.to_string(),
    })
}

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
