use req_core::req::model::{
    Header, HttpMethod, ParsedRequest, Request, RequestBody, RequestOptions, Variable,
};
use req_core::req::resolver::{ResolveContext, needs_context, resolve_request};

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
            options: RequestOptions::default(),
        },
    }
}

fn context(defaults: Vec<(&str, &str)>, envs: Vec<(&str, Vec<(&str, &str)>)>) -> ResolveContext {
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

#[test]
fn rejects_missing_template_variable() {
    let parsed = parsed_request(vec![], vec![("TOKEN", "abc123")], "{{BASE_URL}}/posts");
    let context = context(vec![("USER_ID", "1")], vec![]);

    let error = resolve_request(parsed, context).unwrap_err();

    assert_eq!(error.message, "Missing variable: BASE_URL");
}

#[test]
fn rejects_missing_template_variable_in_header() {
    let parsed = parsed_request(vec![], vec![], "https://example.com/posts");
    let context = context(vec![("USER_ID", "1")], vec![]);

    let error = resolve_request(parsed, context).unwrap_err();

    assert_eq!(error.message, "Missing variable: TOKEN");
}

#[test]
fn rejects_missing_template_variable_in_body() {
    let parsed = parsed_request(
        vec![],
        vec![("TOKEN", "abc123")],
        "https://example.com/posts",
    );
    let context = context(vec![], vec![]);

    let error = resolve_request(parsed, context).unwrap_err();

    assert_eq!(error.message, "Missing variable: USER_ID");
}

#[test]
fn rejects_unclosed_template_variable() {
    let parsed = parsed_request(vec![], vec![("TOKEN", "abc123")], "{{BASE_URL/posts");
    let context = context(vec![("USER_ID", "1")], vec![]);

    let error = resolve_request(parsed, context).unwrap_err();

    assert_eq!(error.message, "Unclosed variable expression");
}

#[test]
fn rejects_empty_template_variable() {
    let parsed = parsed_request(vec![], vec![("TOKEN", "abc123")], "{{ }}");
    let context = context(vec![("USER_ID", "1")], vec![]);

    let error = resolve_request(parsed, context).unwrap_err();

    assert_eq!(error.message, "Empty variable expression");
}

#[test]
fn parses_valid_context_json() {
    let context = ResolveContext::from_json(
        r#"{
          "defaults": {
            "BASE_URL": "https://api.example.com"
          },
          "envs": {
            "dev": {
              "TOKEN": "abc123"
            }
          }
        }"#,
    )
    .unwrap();

    assert_eq!(
        context.defaults.get("BASE_URL"),
        Some(&"https://api.example.com".to_string())
    );
    assert_eq!(
        context.envs.get("dev").and_then(|env| env.get("TOKEN")),
        Some(&"abc123".to_string())
    );
}

#[test]
fn rejects_invalid_context_json() {
    let error = ResolveContext::from_json("{invalid").unwrap_err();

    assert!(error.message.starts_with("Invalid context JSON:"));
}

#[test]
fn rejects_context_json_with_invalid_shape() {
    let error = ResolveContext::from_json(
        r#"{
          "defaults": [],
          "envs": {}
        }"#,
    )
    .unwrap_err();

    assert!(error.message.starts_with("Invalid context JSON:"));
}

#[test]
fn request_without_envs_or_external_vars_does_not_need_context() {
    let parsed = ParsedRequest {
        name: None,
        envs: vec![],
        vars: vec![],
        request: Request {
            method: HttpMethod::Get,
            url: "https://example.com/posts".to_string(),
            headers: vec![Header {
                name: "Accept".to_string(),
                value: "application/json".to_string(),
            }],
            body: None,
            options: RequestOptions::default(),
        },
    };

    assert!(!needs_context(&parsed).unwrap());
}

#[test]
fn request_with_selected_env_needs_context() {
    let parsed = ParsedRequest {
        name: None,
        envs: vec!["dev".to_string()],
        vars: vec![],
        request: Request {
            method: HttpMethod::Get,
            url: "https://example.com/posts".to_string(),
            headers: vec![],
            body: None,
            options: RequestOptions::default(),
        },
    };

    assert!(needs_context(&parsed).unwrap());
}

#[test]
fn request_with_external_template_var_needs_context() {
    let parsed = ParsedRequest {
        name: None,
        envs: vec![],
        vars: vec![],
        request: Request {
            method: HttpMethod::Get,
            url: "{{BASE_URL}}/posts".to_string(),
            headers: vec![],
            body: None,
            options: RequestOptions::default(),
        },
    };

    assert!(needs_context(&parsed).unwrap());
}

#[test]
fn request_with_only_inline_template_vars_does_not_need_context() {
    let parsed = ParsedRequest {
        name: None,
        envs: vec![],
        vars: vec![Variable {
            name: "BASE_URL".to_string(),
            value: "https://example.com".to_string(),
        }],
        request: Request {
            method: HttpMethod::Get,
            url: "{{BASE_URL}}/posts".to_string(),
            headers: vec![],
            body: None,
            options: RequestOptions::default(),
        },
    };

    assert!(!needs_context(&parsed).unwrap());
}
