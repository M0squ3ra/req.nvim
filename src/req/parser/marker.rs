/// Parses a request block marker.
///
/// Returns `None` when the line is not a marker, `Some(None)` for `###`, and
/// `Some(Some(name))` for `### Name`.
pub fn parse_request_marker(line: &str) -> Option<Option<String>> {
    let name = line.strip_prefix("###")?.trim();

    if name.is_empty() {
        return Some(None);
    }

    Some(Some(name.to_string()))
}
