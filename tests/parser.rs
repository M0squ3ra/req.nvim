use req_core::req::lowering::lower_first_request;
use req_core::req::model::{Header, HttpMethod, RequestBody, Variable};
use req_core::req::parser::ast::{Directive, ReqLine};
use req_core::req::parser::parse;

fn parse_request(
    input: &str,
) -> Result<req_core::req::model::ParsedRequest, req_core::req::error::ParseError> {
    let document = parse(input);
    lower_first_request(&document)
}

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
        Some(RequestBody::Raw(
            "# body comment\n{\n  \"ok\": true\n}".to_string()
        ))
    );
}

#[test]
fn document_splits_multiple_request_blocks() {
    let document = parse(
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
fn document_splits_unmarked_request_blocks() {
    let document = parse(
        r#"GET https://example.com/first

# second request
# @env dev
GET https://example.com/second"#,
    );

    assert_eq!(document.requests.len(), 2);
    assert_eq!(document.requests[0].start_line, 1);
    assert_eq!(document.requests[1].start_line, 3);
    assert_eq!(
        document.requests[1].lines[0],
        ReqLine::Comment("# second request".to_string())
    );
    assert_eq!(
        document.requests[1].lines[1],
        ReqLine::Directive(Directive::Env("dev".to_string()))
    );
}

#[test]
fn document_does_not_split_body_method_words_without_url() {
    let document = parse(
        r#"POST https://example.com
Content-Type: text/plain

GET this is body text"#,
    );

    assert_eq!(document.requests.len(), 1);
}

#[test]
fn document_classifies_comments_and_directives() {
    let document = parse(
        r#"### Test
# normal comment
# @env dev
@TOKEN=abc123
GET https://example.com"#,
    );

    let lines = &document.requests[0].lines;

    assert_eq!(lines[0], ReqLine::Comment("# normal comment".to_string()));
    assert_eq!(
        lines[1],
        ReqLine::Directive(Directive::Env("dev".to_string()))
    );
    assert_eq!(
        lines[2],
        ReqLine::Directive(Directive::Variable {
            name: "TOKEN".to_string(),
            value: "abc123".to_string(),
        })
    );
}

#[test]
fn fixture_parses_simple_unmarked_request() {
    let parsed = parse_request(include_str!("fixtures/parser/simple_unmarked.http")).unwrap();

    assert_eq!(parsed.name, None);
    assert_eq!(parsed.request.method, HttpMethod::Get);
    assert_eq!(parsed.request.url, "https://example.com/users");
    assert_eq!(
        parsed.request.headers,
        vec![Header {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        }]
    );
    assert_eq!(parsed.request.body, None);
}

#[test]
fn fixture_parses_marked_request_with_env_and_vars() {
    let parsed = parse_request(include_str!(
        "fixtures/parser/marked_with_env_and_vars.http"
    ))
    .unwrap();

    assert_eq!(parsed.name, Some("Get post".to_string()));
    assert_eq!(parsed.envs, vec!["dev".to_string()]);
    assert_eq!(
        parsed.vars,
        vec![Variable {
            name: "POST_ID".to_string(),
            value: "1".to_string(),
        }]
    );
    assert_eq!(parsed.request.url, "{{BASE_URL}}/posts/{{POST_ID}}");
}

#[test]
fn fixture_parses_post_with_json_body() {
    let parsed = parse_request(include_str!("fixtures/parser/post_with_json_body.http")).unwrap();

    assert_eq!(parsed.name, Some("Create post".to_string()));
    assert_eq!(parsed.request.method, HttpMethod::Post);
    assert_eq!(parsed.request.headers.len(), 2);
    assert_eq!(
        parsed.request.body,
        Some(RequestBody::Raw(
            "{\n  \"title\": \"{{TITLE}}\",\n  \"published\": true\n}".to_string()
        ))
    );
}

#[test]
fn fixture_splits_mixed_multiple_requests() {
    let document = parse(include_str!("fixtures/parser/mixed_multiple_requests.http"));

    assert_eq!(document.requests.len(), 4);
    assert_eq!(document.requests[0].name, None);
    assert_eq!(document.requests[1].name, Some("Create user".to_string()));
    assert_eq!(document.requests[2].name, None);
    assert_eq!(document.requests[3].name, Some("Update user".to_string()));
}

#[test]
fn fixture_does_not_split_body_method_words() {
    let document = parse(include_str!("fixtures/parser/body_with_method_words.http"));

    assert_eq!(document.requests.len(), 1);
}

#[test]
fn fixture_splits_large_mixed_http_file() {
    let document = parse(include_str!("fixtures/large_mixed.http"));

    assert_eq!(document.requests.len(), 8);
    assert_eq!(document.requests[0].name, None);
    assert_eq!(document.requests[1].name, Some("List users".to_string()));
    assert_eq!(document.requests[2].name, None);
    assert_eq!(document.requests[3].name, Some("Create user".to_string()));
    assert_eq!(document.requests[4].name, None);
    assert_eq!(
        document.requests[5].name,
        Some("Ping absolute url".to_string())
    );
    assert_eq!(document.requests[6].name, None);
    assert_eq!(document.requests[7].name, Some("Text payload".to_string()));
}

#[test]
fn fixture_large_mixed_first_request_keeps_prelude() {
    let document = parse(include_str!("fixtures/large_mixed.http"));
    let first = &document.requests[0];

    assert_eq!(first.start_line, 1);
    assert_eq!(
        first.lines[0],
        ReqLine::Comment("# workspace comment".to_string())
    );
    assert_eq!(
        first.lines[1],
        ReqLine::Directive(Directive::Env("dev".to_string()))
    );
    assert_eq!(
        first.lines[2],
        ReqLine::Directive(Directive::Variable {
            name: "API_VERSION".to_string(),
            value: "v1".to_string(),
        })
    );
}

#[test]
fn fixture_large_mixed_parses_body_with_method_words() {
    let parsed = parse_request(
        r#"### Text payload
POST {{BASE_URL}}/logs
Content-Type: text/plain

GET this line is part of the body
POST this one too"#,
    )
    .unwrap();

    assert_eq!(
        parsed.request.body,
        Some(RequestBody::Raw(
            "GET this line is part of the body\nPOST this one too".to_string()
        ))
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
fn rejects_missing_url() {
    let error = parse_request("GET").unwrap_err();

    assert_eq!(error.message, "Missing url");
    assert_eq!(error.line, 1);
    assert_eq!(error.column, 5);
}

#[test]
fn rejects_invalid_request_line_with_extra_parts() {
    let error = parse_request("GET https://example.com extra").unwrap_err();

    assert_eq!(error.message, "Invalid request line");
    assert_eq!(error.line, 1);
}

#[test]
fn rejects_unsupported_method() {
    let error = parse_request("TRACE https://example.com").unwrap_err();

    assert_eq!(error.message, "Unsupported method: TRACE");
    assert_eq!(error.line, 1);
    assert_eq!(error.column, 1);
}

#[test]
fn rejects_missing_header_name() {
    let error = parse_request(
        r#"GET https://example.com
: missing"#,
    )
    .unwrap_err();

    assert_eq!(error.message, "Missing header name");
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
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

#[test]
fn preserves_comment_lines_inside_body() {
    let parsed = parse_request(
        r#"POST https://example.com/logs
Content-Type: text/plain

# body comment
// another body line
plain text"#,
    )
    .unwrap();

    assert_eq!(
        parsed.request.body,
        Some(RequestBody::Raw(
            "# body comment\n// another body line\nplain text".to_string()
        ))
    );
}

#[test]
fn document_splits_unmarked_relative_url_requests() {
    let document = parse(
        r#"GET /first

# second request
GET /second"#,
    );

    assert_eq!(document.requests.len(), 2);
    assert_eq!(document.requests[0].start_line, 1);
    assert_eq!(document.requests[1].start_line, 3);
}
