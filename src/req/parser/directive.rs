use super::ast::Directive;

/// Parses a `.req` directive line.
///
pub fn parse_directive(line: &str) -> Option<Directive> {
    let directive = line.strip_prefix("@")?;
    let mut parts = directive.split_whitespace();
    let name = parts.next()?;

    match name {
        "env" => parse_env(parts),
        _ => None,
    }
}

/// Parses the arguments of an `@env` directive.
fn parse_env<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Directive> {
    let env = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    Some(Directive::Env(env.to_string()))
}
