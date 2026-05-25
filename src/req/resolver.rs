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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::req::model::{Header, HttpMethod};

    fn parsed_request(envs: Vec<&str>, vars: Vec<(&str, &str)>, url: &str) -> ParsedRequest {
        ParsedRequest {
            name: None,
            envs: envs.into_iter().map(String::from).collect(),
            vars: vars
                .into_iter()
                .map(|(name, value)| Variable {
                    name: name.to_string(),
                    value: value.to_string(),
                })
                .collect(),
            request: Request {
                method: HttpMethod::Get,
                url: url.to_string(),
                headers: vec![Header {
                    name: "Authorization".to_string(),
                    value: "Bearer {{TOKEN}}".to_string(),
                }],
                body: Some(RequestBody::Raw("{\"id\": {{USER_ID}}}".to_string())),
            },
        }
    }

    fn context(
        defaults: Vec<(&str, &str)>,
        envs: Vec<(&str, Vec<(&str, &str)>)>,
    ) -> ResolveContext {
        ResolveContext {
            defaults: defaults
                .into_iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            envs: envs
                .into_iter()
                .map(|(env_name, vars)| {
                    (
                        env_name.to_string(),
                        vars.into_iter()
                            .map(|(name, value)| (name.to_string(), value.to_string()))
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn merges_multiple_selected_envs() {
        let parsed = parsed_request(
            vec!["dev", "auth"],
            vec![],
            "{{BASE_URL}}/posts/{{USER_ID}}",
        );
        let context = context(
            vec![],
            vec![
                ("dev", vec![("BASE_URL", "https://dev.example.com")]),
                ("auth", vec![("TOKEN", "abc123"), ("USER_ID", "1")]),
            ],
        );

        let request = resolve_request(parsed, context).unwrap();

        assert_eq!(request.url, "https://dev.example.com/posts/1");
        assert_eq!(request.headers[0].value, "Bearer abc123");
        assert_eq!(
            request.body,
            Some(RequestBody::Raw("{\"id\": 1}".to_string()))
        );
    }

    #[test]
    fn rejects_colliding_selected_envs() {
        let parsed = parsed_request(vec!["dev", "staging"], vec![], "{{BASE_URL}}/posts");
        let context = context(
            vec![],
            vec![
                ("dev", vec![("BASE_URL", "https://dev.example.com")]),
                ("staging", vec![("BASE_URL", "https://staging.example.com")]),
            ],
        );

        let error = resolve_request(parsed, context).unwrap_err();

        assert_eq!(
            error.message,
            "Variable BASE_URL is defined by multiple selected environments: dev, staging"
        );
    }

    #[test]
    fn inline_vars_override_env_vars() {
        let parsed = parsed_request(
            vec!["dev", "auth"],
            vec![("BASE_URL", "https://local.example.com")],
            "{{BASE_URL}}/posts/{{USER_ID}}",
        );
        let context = context(
            vec![],
            vec![
                ("dev", vec![("BASE_URL", "https://dev.example.com")]),
                ("auth", vec![("TOKEN", "abc123"), ("USER_ID", "1")]),
            ],
        );

        let request = resolve_request(parsed, context).unwrap();

        assert_eq!(request.url, "https://local.example.com/posts/1");
    }

    #[test]
    fn env_vars_override_defaults() {
        let parsed = parsed_request(vec!["dev", "auth"], vec![], "{{BASE_URL}}/posts");
        let context = context(
            vec![("BASE_URL", "https://api.example.com")],
            vec![
                ("dev", vec![("BASE_URL", "https://dev.example.com")]),
                ("auth", vec![("TOKEN", "abc123"), ("USER_ID", "1")]),
            ],
        );

        let request = resolve_request(parsed, context).unwrap();

        assert_eq!(request.url, "https://dev.example.com/posts");
    }

    #[test]
    fn rejects_missing_selected_env() {
        let parsed = parsed_request(vec!["missing"], vec![], "{{BASE_URL}}/posts");
        let context = context(vec![], vec![]);

        let error = resolve_request(parsed, context).unwrap_err();

        assert_eq!(error.message, "Missing environment: missing");
    }

    #[test]
    fn deduplicates_selected_envs() {
        let parsed = parsed_request(vec!["dev", "dev"], vec![], "{{BASE_URL}}/posts");
        let context = context(
            vec![("TOKEN", "abc123"), ("USER_ID", "1")],
            vec![("dev", vec![("BASE_URL", "https://dev.example.com")])],
        );

        let request = resolve_request(parsed, context).unwrap();

        assert_eq!(request.url, "https://dev.example.com/posts");
    }

    #[test]
    fn rejects_duplicated_inline_vars() {
        let parsed = parsed_request(
            vec!["auth"],
            vec![("BASE_URL", "one"), ("BASE_URL", "two")],
            "{{BASE_URL}}/posts",
        );
        let context = context(vec![], vec![("auth", vec![("TOKEN", "abc123")])]);

        let error = resolve_request(parsed, context).unwrap_err();

        assert_eq!(error.message, "Duplicated variable: BASE_URL");
    }
}
