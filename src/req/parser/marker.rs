pub fn parse_request_marker(line: &str) -> Option<Option<String>> {
    let name = line.strip_prefix("###")?.trim();

    if name.is_empty() {
        return Some(None);
    }

    Some(Some(name.to_string()))
}
