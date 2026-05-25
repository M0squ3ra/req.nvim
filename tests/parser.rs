use req_core::req::model::{Header, HttpMethod, RequestBody, Variable};
use req_core::req::parser::ast::{Directive, ReqLine};
use req_core::req::parser::{parse_document, parse_request};

#[test]
fn parses_minimal_request() {
    let parsed = parse_request("GET https://example.com").unwrap();

    assert_eq!(parsed.request.method, HttpMethod::Get);
    assert_eq!(parsed.request.url, "https://example.com");
    assert!(parsed.request.headers.is_empty());
    assert_eq!(parsed.request.body, None);
}

#[test]
fn parses_request_name() {
    let parsed = parse_request(
        r#"### Get user
GET https://example.com/users/1"#,
    )
    .unwrap();

    assert_eq!(parsed.name, Some("Get user".to_string()));
}

#[test]
fn parses_env_metadata_comments() {
    let parsed = parse_request(
        r#"### Get user
# @env dev
# @env auth
GET https://example.com/users/1"#,
    )
    .unwrap();

    assert_eq!(parsed.envs, vec!["dev".to_string(), "auth".to_string()]);
}

#[test]
fn parses_inline_variables() {
    let parsed = parse_request(
        r#"@BASE_URL=https://example.com
@TOKEN=abc123
GET {{BASE_URL}}/users"#,
    )
    .unwrap();

    assert_eq!(
        parsed.vars,
        vec![
            Variable {
                name: "BASE_URL".to_string(),
                value: "https://example.com".to_string(),
            },
            Variable {
                name: "TOKEN".to_string(),
                value: "abc123".to_string(),
            },
        ]
    );
}

#[test]
fn parses_headers() {
    let parsed = parse_request(
        r#"GET https://example.com/users
Accept: application/json
Authorization: Bearer token"#,
    )
    .unwrap();

    assert_eq!(
        parsed.request.headers,
        vec![
            Header {
                name: "Accept".to_string(),
                value: "application/json".to_string(),
            },
            Header {
                name: "Authorization".to_string(),
                value: "Bearer token".to_string(),
            },
        ]
    );
}

#[test]
fn parses_body_after_empty_line() {
    let parsed = parse_request(
        r#"POST https://example.com/users
Content-Type: application/json

{
  "name": "John"
}"#,
    )
    .unwrap();

    assert_eq!(
        parsed.request.body,
        Some(RequestBody::Raw("{\n  \"name\": \"John\"\n}".to_string()))
    );
}

#[test]
fn ignores_regular_comments() {
    let parsed = parse_request(
        r#"# top comment
// another comment
# @env dev
@TOKEN=abc123
GET https://example.com/users
# header comment
Accept: application/json

# body comment
{
  "ok": true
}"#,
    )
    .unwrap();

    assert_eq!(parsed.envs, vec!["dev".to_string()]);
    assert_eq!(parsed.vars[0].name, "TOKEN");
    assert_eq!(parsed.request.headers.len(), 1);
    assert_eq!(
        parsed.request.body,
        Some(RequestBody::Raw("{\n  \"ok\": true\n}".to_string()))
    );
}

#[test]
fn document_splits_multiple_request_blocks() {
    let document = parse_document(
        r#"### First
GET https://example.com/first

### Second
GET https://example.com/second"#,
    );

    assert_eq!(document.requests.len(), 2);
    assert_eq!(document.requests[0].name, Some("First".to_string()));
    assert_eq!(document.requests[1].name, Some("Second".to_string()));
}

#[test]
fn document_classifies_comments_and_directives() {
    let document = parse_document(
        r#"### Test
# normal comment
# @env dev
@TOKEN=abc123
GET https://example.com"#,
    );

    let lines = &document.requests[0].lines;

    assert_eq!(lines[0], ReqLine::Comment);
    assert_eq!(lines[1], ReqLine::Directive(Directive::Env("dev".to_string())));
    assert_eq!(
        lines[2],
        ReqLine::Directive(Directive::Variable {
            name: "TOKEN".to_string(),
            value: "abc123".to_string(),
        })
    );
}

#[test]
fn rejects_missing_request_line() {
    let error = parse_request(
        r#"### Empty
# @env dev"#,
    )
    .unwrap_err();

    assert_eq!(error.message, "Missing request line");
}

#[test]
fn rejects_invalid_header() {
    let error = parse_request(
        r#"GET https://example.com
Invalid header"#,
    )
    .unwrap_err();

    assert_eq!(error.message, "Invalid header");
}

#[test]
fn rejects_directive_after_request_line() {
    let error = parse_request(
        r#"GET https://example.com
@TOKEN=abc123"#,
    )
    .unwrap_err();

    assert_eq!(error.message, "Directive must appear before request line");
}
