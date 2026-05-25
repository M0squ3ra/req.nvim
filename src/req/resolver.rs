use std::collections::HashMap;

use super::model::{ParsedRequest, Request, RequestBody, Variable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub message: String,
}

/// Applies request-local variables to the parsed request.
pub fn resolve_request(parsed: ParsedRequest) -> Result<Request, ResolveError> {
    let vars = collect_vars(parsed.vars)?;
    let mut request = parsed.request;

    request.url = resolve_template(&request.url, &vars)?;

    for header in &mut request.headers {
        header.value = resolve_template(&header.value, &vars)?;
    }

    if let Some(RequestBody::Raw(body)) = request.body {
        request.body = Some(RequestBody::Raw(resolve_template(&body, &vars)?));
    }

    Ok(request)
}

fn collect_vars(vars: Vec<Variable>) -> Result<HashMap<String, String>, ResolveError> {
    let mut map = HashMap::new();

    for var in vars {
        if map.contains_key(&var.name) {
            return Err(ResolveError {
                message: format!("Duplicated variable: {}", var.name),
            });
        }

        map.insert(var.name, var.value);
    }

    Ok(map)
}

fn resolve_template(
    input: &str,
    vars: &HashMap<String, String>,
) -> Result<String, ResolveError> {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];

        let Some(end) = after_start.find("}}") else {
            return Err(ResolveError {
                message: "Unclosed variable expression".to_string(),
            });
        };

        let name = after_start[..end].trim();

        if name.is_empty() {
            return Err(ResolveError {
                message: "Empty variable expression".to_string(),
            });
        }

        let Some(value) = vars.get(name) else {
            return Err(ResolveError {
                message: format!("Missing variable: {}", name),
            });
        };

        output.push_str(value);
        rest = &after_start[end + 2..];
    }

    output.push_str(rest);
    Ok(output)
}
