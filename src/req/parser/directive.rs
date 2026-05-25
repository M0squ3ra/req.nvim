use super::ast::Directive;

/// Parses a `.req` directive line.
/// Currently supports `@env name`, `@var NAME=value`, and `@body`.
pub fn parse_directive(line: &str) -> Option<Directive> {
    let directive = line.strip_prefix("@")?;
    let mut parts = directive.split_whitespace();
    let name = parts.next()?;

    match name {
        "env" => parse_env(parts),
        "var" => parse_var(parts),
        "body" => parse_body(parts),
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

/// Parses the arguments of an `@var` directive.
fn parse_var<'a>(parts: impl Iterator<Item = &'a str>) -> Option<Directive> {
    let variable = parts.collect::<Vec<_>>().join(" ");
    let (name, value) = variable.split_once("=")?;
    let name = name.trim();
    let value = value.trim();

    if name.is_empty() || value.is_empty() {
        return None;
    }

    Some(Directive::Var {
        name: name.to_string(),
        value: value.to_string(),
    })
}

/// Parses the arguments of an `@body` directive.
fn parse_body<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Directive> {
    if parts.next().is_some() {
        return None;
    }

    Some(Directive::Body)
}
