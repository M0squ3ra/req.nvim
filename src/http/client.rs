use crate::req::model::{Header, HttpMethod, Request, RequestBody, Response};

pub fn execute(request: Request) -> Result<Response, String> {
    let client = reqwest::blocking::Client::new();
    let method = to_reqwest_method(request.method);
    let mut builder = client.request(method, request.url);

    for header in request.headers {
        builder = builder.header(header.name, header.value);
    }

    if let Some(RequestBody::Raw(body)) = request.body {
        builder = builder.body(body);
    }

    let response = builder.send().map_err(|error| error.to_string())?;

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
