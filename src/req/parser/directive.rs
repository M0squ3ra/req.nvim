pub fn parse_env_directive(line: &str) -> Option<String> {
    let env = line.strip_prefix("@env ")?.trim();

    if env.is_empty() {
        return None;
    }

    Some(env.to_string())
}
