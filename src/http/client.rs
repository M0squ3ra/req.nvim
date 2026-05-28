use std::time::Duration;

use crate::req::model::{Header, HttpMethod, Request, RequestBody, Response};

pub fn execute(request: Request) -> Result<Response, String> {
    let client = client(&request)?;
    let timeout_ms = request.options.timeout_ms;
    let method = to_reqwest_method(request.method);
    let mut builder = client.request(method, request.url);

    for header in request.headers {
        builder = builder.header(header.name, header.value);
    }

    if let Some(RequestBody::Raw(body)) = request.body {
        builder = builder.body(body);
    }

    let response = builder
        .send()
        .map_err(|error| request_error(error, timeout_ms))?;

    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| Header {
            name: name.to_string(),
            value: value.to_str().unwrap_or("").to_string(),
        })
        .collect();

    let body = response.text().map_err(|error| error.to_string())?;
    Ok(Response {
        status,
        headers,
        body,
    })
}

fn request_error(error: reqwest::Error, timeout_ms: Option<u64>) -> String {
    if error.is_timeout() {
        if let Some(timeout_ms) = timeout_ms {
            return format!("request timed out after {timeout_ms}ms");
        }

        return "request timed out".to_string();
    }

    error.to_string()
}

fn client(request: &Request) -> Result<reqwest::blocking::Client, String> {
    let mut builder = reqwest::blocking::Client::builder();

    if let Some(timeout_ms) = request.options.timeout_ms {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }

    builder.build().map_err(|error| error.to_string())
}

fn to_reqwest_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    }
}
