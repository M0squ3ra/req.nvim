use super::ast::Directive;

/// Parses a `.req` directive line.
/// Currently supports `# @env name` and `.http`-style variables like `@NAME=value`.
pub fn parse_directive(line: &str) -> Option<Directive> {
    parse_metadata_comment(line).or_else(|| parse_inline_variable(line))
}

/// Parses a `.http` metadata comment supported by req.nvim.
fn parse_metadata_comment(line: &str) -> Option<Directive> {
    let metadata = line.strip_prefix("#")?.trim();
    let mut parts = metadata.split_whitespace();
    let name = parts.next()?;

    match name {
        "@env" => parse_env(parts),
        _ => None,
    }
}

/// Parses a `.http`-style inline variable.
fn parse_inline_variable(line: &str) -> Option<Directive> {
    let directive = line.strip_prefix("@")?;
    parse_variable(directive)
}

/// Parses the arguments of an `@env` directive.
fn parse_env<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Directive> {
    let env = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    Some(Directive::Env(env.to_string()))
}

/// Parses a `.http`-style inline variable.
fn parse_variable(variable: &str) -> Option<Directive> {
    let (name, value) = variable.split_once("=")?;
    let name = name.trim();
    let value = value.trim();

    if name.is_empty() || value.is_empty() {
        return None;
    }

    Some(Directive::Variable {
        name: name.to_string(),
        value: value.to_string(),
    })
}
