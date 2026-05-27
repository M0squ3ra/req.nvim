use serde::Serialize;

#[derive(Serialize)]
pub struct RequestListOutput {
    pub requests: Vec<req_core::req::listing::RequestListing>,
}

#[derive(Serialize)]
pub struct CheckOutput {
    pub ok: bool,
}

#[derive(Serialize)]
pub struct CurlOutput {
    pub command: String,
}

#[derive(Serialize)]
pub struct ExecutedResponseOutput {
    pub request: ExecutedRequestOutput,
    pub response: ResponseOutput,
}

#[derive(Serialize)]
pub struct ExecutedRequestOutput {
    pub name: String,
    pub directives: Vec<String>,
    pub method: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct ResponseOutput {
    pub status: u16,
    pub headers: Vec<HeaderOutput>,
    pub body: String,
}

#[derive(Serialize)]
pub struct HeaderOutput {
    pub name: String,
    pub value: String,
}

impl From<req_core::req::model::Response> for ResponseOutput {
    fn from(response: req_core::req::model::Response) -> Self {
        Self {
            status: response.status,
            headers: response
                .headers
                .into_iter()
                .map(HeaderOutput::from)
                .collect(),
            body: response.body,
        }
    }
}

impl From<req_core::req::model::Header> for HeaderOutput {
    fn from(header: req_core::req::model::Header) -> Self {
        Self {
            name: header.name,
            value: header.value,
        }
    }
}
