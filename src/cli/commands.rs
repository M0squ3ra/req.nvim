use super::{
    args::{CliOptions, Mode},
    dto::{
        CheckOutput, CurlOutput, ExecutedRequestOutput, ExecutedResponseOutput, RequestListOutput,
    },
};

pub enum CommandOutput {
    RequestList(RequestListOutput),
    Check(CheckOutput),
    Curl(CurlOutput),
    ExecutedResponse(ExecutedResponseOutput),
}

pub fn run_command(options: &CliOptions, input: &str) -> Result<CommandOutput, String> {
    match options.mode {
        Mode::ListRequests => list_requests(input),
        Mode::Check => check_request(input, options.context_json.as_deref()),
        Mode::ExportCurl => export_curl(input, options.context_json.as_deref()),
        Mode::Run => execute_request(input, options.context_json.as_deref()),
    }
}

fn list_requests(input: &str) -> Result<CommandOutput, String> {
    Ok(CommandOutput::RequestList(RequestListOutput {
        requests: req_core::req::listing::list_requests(input),
    }))
}

fn export_curl(input: &str, context_json: Option<&str>) -> Result<CommandOutput, String> {
    let resolved = resolve_selected_request(input, context_json)?;

    Ok(CommandOutput::Curl(CurlOutput {
        command: req_core::req::curl::to_curl(&resolved.request),
    }))
}

fn check_request(input: &str, context_json: Option<&str>) -> Result<CommandOutput, String> {
    resolve_selected_request(input, context_json)?;

    Ok(CommandOutput::Check(CheckOutput { ok: true }))
}

fn execute_request(input: &str, context_json: Option<&str>) -> Result<CommandOutput, String> {
    let resolved = resolve_selected_request(input, context_json)?;
    let request = ExecutedRequestOutput {
        name: resolved.name,
        directives: resolved.directives,
        method: method_as_str(&resolved.request.method).to_string(),
        url: resolved.request.url.clone(),
    };
    let response = req_core::http::client::execute(resolved.request)
        .map_err(|error| format!("request error: {error}"))?;

    Ok(CommandOutput::ExecutedResponse(ExecutedResponseOutput {
        request,
        response: response.into(),
    }))
}

struct ResolvedRequest {
    name: String,
    directives: Vec<String>,
    request: req_core::req::model::Request,
}

fn resolve_selected_request(
    input: &str,
    context_json: Option<&str>,
) -> Result<ResolvedRequest, String> {
    let parsed_request = parse_selected_request(input)?;

    let request_name = parsed_request
        .name
        .clone()
        .unwrap_or_else(|| "Untitled request".to_string());
    let directives = parsed_request
        .envs
        .iter()
        .map(|env| format!("@env {}", env))
        .chain(
            parsed_request
                .vars
                .iter()
                .map(|var| format!("@{}={}", var.name, var.value)),
        )
        .collect::<Vec<_>>();

    let context = context_for_request(&parsed_request, context_json)
        .map_err(|error| format!("Resolve error: {}", error.message))?;
    let request = req_core::req::resolver::resolve_request(parsed_request, context)
        .map_err(|error| format!("Resolve error: {}", error.message))?;

    Ok(ResolvedRequest {
        name: request_name,
        directives,
        request,
    })
}

fn parse_selected_request(input: &str) -> Result<req_core::req::model::ParsedRequest, String> {
    let document = req_core::req::parser::parse(input);
    req_core::req::lowering::lower_first_request(&document).map_err(|error| {
        format!(
            "Parse error at {}:{}: {}",
            error.line, error.column, error.message
        )
    })
}

fn context_for_request(
    parsed_request: &req_core::req::model::ParsedRequest,
    context_json: Option<&str>,
) -> Result<req_core::req::resolver::ResolveContext, req_core::req::resolver::ResolveError> {
    let Some(context_json) = context_json else {
        return Ok(Default::default());
    };

    let needs_context = req_core::req::resolver::needs_context(parsed_request)?;

    match req_core::req::resolver::ResolveContext::from_json(context_json) {
        Ok(context) => Ok(context),
        Err(error) if !needs_context => {
            eprintln!("Warning: {}", error.message);
            Ok(Default::default())
        }
        Err(error) => Err(error),
    }
}

fn method_as_str(method: &req_core::req::model::HttpMethod) -> &'static str {
    match method {
        req_core::req::model::HttpMethod::Get => "GET",
        req_core::req::model::HttpMethod::Post => "POST",
        req_core::req::model::HttpMethod::Put => "PUT",
        req_core::req::model::HttpMethod::Patch => "PATCH",
        req_core::req::model::HttpMethod::Delete => "DELETE",
        req_core::req::model::HttpMethod::Head => "HEAD",
        req_core::req::model::HttpMethod::Options => "OPTIONS",
    }
}
