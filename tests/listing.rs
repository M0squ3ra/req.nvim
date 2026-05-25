use req_core::req::listing::{list_requests, RequestListing};

#[test]
fn lists_request_ranges_with_markers() {
    let requests = list_requests(
        r#"### First
GET https://example.com/first

### Second
# @env dev
GET https://example.com/second
Accept: application/json"#,
    );

    assert_eq!(
        requests,
        vec![
            RequestListing {
                name: Some("First".to_string()),
                start_line: 1,
                end_line: 3,
            },
            RequestListing {
                name: Some("Second".to_string()),
                start_line: 4,
                end_line: 7,
            },
        ]
    );
}

#[test]
fn lists_single_unmarked_request() {
    let requests = list_requests(
        r#"GET https://example.com
Accept: application/json"#,
    );

    assert_eq!(
        requests,
        vec![RequestListing {
            name: None,
            start_line: 1,
            end_line: 2,
        }]
    );
}

#[test]
fn lists_multiple_unmarked_requests() {
    let requests = list_requests(
        r#"GET https://example.com/first
Accept: application/json

# second request
# @env dev
@TOKEN=abc123
POST https://example.com/second
Content-Type: application/json

{
  "ok": true
}"#,
    );

    assert_eq!(
        requests,
        vec![
            RequestListing {
                name: None,
                start_line: 1,
                end_line: 3,
            },
            RequestListing {
                name: None,
                start_line: 4,
                end_line: 12,
            },
        ]
    );
}

#[test]
fn does_not_split_body_text_that_starts_with_method_word() {
    let requests = list_requests(
        r#"POST https://example.com
Content-Type: text/plain

GET this is body text"#,
    );

    assert_eq!(
        requests,
        vec![RequestListing {
            name: None,
            start_line: 1,
            end_line: 4,
        }]
    );
}
