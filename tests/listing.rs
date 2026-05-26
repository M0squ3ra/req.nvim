use req_core::req::listing::{RequestListing, list_requests};

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

#[test]
fn fixture_lists_mixed_request_ranges() {
    let requests = list_requests(include_str!(
        "fixtures/listing/mixed_multiple_requests.http"
    ));

    assert_eq!(
        requests,
        vec![
            RequestListing {
                name: None,
                start_line: 1,
                end_line: 5,
            },
            RequestListing {
                name: Some("Create user".to_string()),
                start_line: 6,
                end_line: 16,
            },
            RequestListing {
                name: None,
                start_line: 17,
                end_line: 21,
            },
            RequestListing {
                name: Some("Update user".to_string()),
                start_line: 22,
                end_line: 28,
            },
        ]
    );
}

#[test]
fn fixture_lists_large_mixed_request_ranges() {
    let requests = list_requests(include_str!("fixtures/large_mixed.http"));

    assert_eq!(
        requests,
        vec![
            RequestListing {
                name: None,
                start_line: 1,
                end_line: 6,
            },
            RequestListing {
                name: Some("List users".to_string()),
                start_line: 7,
                end_line: 10,
            },
            RequestListing {
                name: None,
                start_line: 11,
                end_line: 13,
            },
            RequestListing {
                name: Some("Create user".to_string()),
                start_line: 14,
                end_line: 25,
            },
            RequestListing {
                name: None,
                start_line: 26,
                end_line: 32,
            },
            RequestListing {
                name: Some("Ping absolute url".to_string()),
                start_line: 33,
                end_line: 35,
            },
            RequestListing {
                name: None,
                start_line: 36,
                end_line: 38,
            },
            RequestListing {
                name: Some("Text payload".to_string()),
                start_line: 39,
                end_line: 44,
            },
        ]
    );
}
