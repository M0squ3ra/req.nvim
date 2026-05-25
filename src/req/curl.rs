use super::model::{HttpMethod, Request, RequestBody};

pub fn to_curl(request: &Request) -> String {
    let mut lines = Vec::new();
    let mut first_line = "curl".to_string();

    if request.method != HttpMethod::Get {
        first_line.push_str(" -X ");
        first_line.push_str(method_as_str(&request.method));
    }

    first_line.push(' ');
    first_line.push_str(&shell_quote(&request.url));
    lines.push(first_line);

    for header in &request.headers {
        lines.push(format!(
            "  -H {}",
            shell_quote(&format!("{}: {}", header.name, header.value))
        ));
    }

    if let Some(RequestBody::Raw(body)) = &request.body {
        lines.push(format!("  --data-raw {}", shell_quote(body)));
    }

    lines.join(" \\\n")
}

fn method_as_str(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Head => "HEAD",
        HttpMethod::Options => "OPTIONS",
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}
