use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use super::model::{ParsedRequest, Request, RequestBody, Variable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ResolveContext {
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    #[serde(default)]
    pub envs: HashMap<String, HashMap<String, String>>,
}

impl ResolveContext {
    pub fn from_json(input: &str) -> Result<Self, ResolveError> {
        serde_json::from_str(input).map_err(|error| ResolveError {
            message: format!("Invalid context JSON: {}", error),
        })
    }
}

/// Returns true when the request needs external context from `--context-json`.
pub fn needs_context(parsed: &ParsedRequest) -> Result<bool, ResolveError> {
    if !parsed.envs.is_empty() {
        return Ok(true);
    }

    let inline_vars = parsed
        .vars
        .iter()
        .map(|var| var.name.as_str())
        .collect::<HashSet<_>>();
    let mut template_vars = HashSet::new();

    collect_template_vars(&parsed.request.url, &mut template_vars)?;

    for header in &parsed.request.headers {
        collect_template_vars(&header.value, &mut template_vars)?;
    }

    if let Some(RequestBody::Raw(body)) = &parsed.request.body {
        collect_template_vars(body, &mut template_vars)?;
    }

    Ok(template_vars
        .into_iter()
        .any(|name| !inline_vars.contains(name.as_str())))
}

/// Applies context variables and request-local variables to the parsed request.
pub fn resolve_request(
    parsed: ParsedRequest,
    context: ResolveContext,
) -> Result<Request, ResolveError> {
    let vars = resolve_vars(&parsed, context)?;
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

fn collect_template_vars(input: &str, vars: &mut HashSet<String>) -> Result<(), ResolveError> {
    let mut rest = input;

    while let Some(start) = rest.find("{{") {
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

        vars.insert(name.to_string());
        rest = &after_start[end + 2..];
    }

    Ok(())
}

fn resolve_vars(
    parsed: &ParsedRequest,
    context: ResolveContext,
) -> Result<HashMap<String, String>, ResolveError> {
    let mut vars = context.defaults;
    let env_vars = collect_env_vars(&parsed.envs, context.envs)?;
    vars.extend(env_vars);
    apply_inline_vars(&mut vars, &parsed.vars)?;

    Ok(vars)
}

fn collect_env_vars(
    selected_envs: &[String],
    envs: HashMap<String, HashMap<String, String>>,
) -> Result<HashMap<String, String>, ResolveError> {
    let mut values = HashMap::new();
    let mut sources = HashMap::new();
    let mut selected = HashSet::new();

    for env_name in selected_envs {
        if !selected.insert(env_name) {
            continue;
        }

        let env = envs.get(env_name).ok_or_else(|| ResolveError {
            message: format!("Missing environment: {}", env_name),
        })?;

        for (name, value) in env {
            if let Some(previous_env) = sources.get(name) {
                return Err(ResolveError {
                    message: format!(
                        "Variable {} is defined by multiple selected environments: {}, {}",
                        name, previous_env, env_name
                    ),
                });
            }

            sources.insert(name.clone(), env_name.clone());
            values.insert(name.clone(), value.clone());
        }
    }

    Ok(values)
}

fn apply_inline_vars(
    vars: &mut HashMap<String, String>,
    inline_vars: &[Variable],
) -> Result<(), ResolveError> {
    let mut seen = HashMap::new();

    for var in inline_vars {
        if seen.contains_key(&var.name) {
            return Err(ResolveError {
                message: format!("Duplicated variable: {}", var.name),
            });
        }

        seen.insert(var.name.clone(), true);
        vars.insert(var.name.clone(), var.value.clone());
    }

    Ok(())
}

fn resolve_template(input: &str, vars: &HashMap<String, String>) -> Result<String, ResolveError> {
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
