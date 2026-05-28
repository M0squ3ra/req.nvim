use req_core::req::curl::to_curl;
use req_core::req::model::{Header, HttpMethod, Request, RequestBody, RequestOptions};

fn request(method: HttpMethod, url: &str) -> Request {
    Request {
        method,
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        options: RequestOptions::default(),
    }
}

#[test]
fn exports_simple_get() {
    let request = request(HttpMethod::Get, "https://example.com/users");

    let curl = to_curl(&request);

    assert_eq!(curl, "curl 'https://example.com/users'");
}

#[test]
fn does_not_add_explicit_get_method() {
    let request = request(HttpMethod::Get, "https://example.com/users");

    let curl = to_curl(&request);

    assert!(!curl.contains("-X GET"));
}

#[test]
fn exports_post_with_headers_and_body() {
    let mut request = request(HttpMethod::Post, "https://example.com/users");
    request.headers = vec![
        Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        Header {
            name: "Authorization".to_string(),
            value: "Bearer abc123".to_string(),
        },
    ];
    request.body = Some(RequestBody::Raw("{\n  \"name\": \"John\"\n}".to_string()));

    let curl = to_curl(&request);

    assert_eq!(
        curl,
        "curl -X POST 'https://example.com/users' \\\n  -H 'Content-Type: application/json' \\\n  -H 'Authorization: Bearer abc123' \\\n  --data-raw '{\n  \"name\": \"John\"\n}'"
    );
}

#[test]
fn escapes_single_quotes() {
    let mut request = request(HttpMethod::Post, "https://example.com/users?name=John's");
    request.headers = vec![Header {
        name: "X-Name".to_string(),
        value: "John's token".to_string(),
    }];
    request.body = Some(RequestBody::Raw("{\"name\":\"John's\"}".to_string()));

    let curl = to_curl(&request);

    assert_eq!(
        curl,
        "curl -X POST 'https://example.com/users?name=John'\\''s' \\\n  -H 'X-Name: John'\\''s token' \\\n  --data-raw '{\"name\":\"John'\\''s\"}'"
    );
}
