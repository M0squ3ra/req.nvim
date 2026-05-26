/// Returns true when a raw line can start a request block.
pub(super) fn looks_like_request_line(line: &str) -> bool {
    let mut parts = line.split_whitespace();
    let Some(method) = parts.next() else {
        return false;
    };
    let Some(url) = parts.next() else {
        return false;
    };

    parts.next().is_none() && is_http_method(method) && looks_like_url(url)
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
        || url.starts_with('/')
}
