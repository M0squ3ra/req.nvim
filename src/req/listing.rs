use serde::Serialize;

use super::parser::parse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestListing {
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn list_requests(input: &str) -> Vec<RequestListing> {
    parse(input)
        .requests
        .into_iter()
        .map(|block| {
            let start_line = block.marker_line.unwrap_or(block.start_line);
            let end_line = if block.lines.is_empty() {
                start_line
            } else {
                block.start_line + block.lines.len() - 1
            };

            RequestListing {
                name: block.name,
                start_line,
                end_line,
            }
        })
        .collect()
}
