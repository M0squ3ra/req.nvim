use std::{
    env,
    io::{self, Read},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args()?;
    let input = read_stdin()?;

    match options.mode {
        Mode::ListRequests => list_requests(&input),
        Mode::ExportCurl => export_curl(&input, options.context_json.as_deref()),
        Mode::Run => execute_request(&input, options.context_json.as_deref()),
    }
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("Failed to read stdin: {error}"))?;

    Ok(input)
}

fn list_requests(input: &str) -> Result<(), String> {
    let json = serde_json::to_string(&req_core::req::listing::list_requests(input))
        .map_err(|error| format!("List requests error: {error}"))?;

    println!("{}", json);
    Ok(())
}

fn export_curl(input: &str, context_json: Option<&str>) -> Result<(), String> {
    let resolved = resolve_selected_request(input, context_json)?;

    println!("{}", req_core::req::curl::to_curl(&resolved.request));
    Ok(())
}

fn execute_request(input: &str, context_json: Option<&str>) -> Result<(), String> {
    let resolved = resolve_selected_request(input, context_json)?;
    let response = req_core::http::client::execute(resolved.request)
        .map_err(|error| format!("request error: {error}"))?;

    print_response(&resolved.name, &resolved.directives, response);
    Ok(())
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

fn print_response(
    request_name: &str,
    directives: &[String],
    response: req_core::req::model::Response,
) {
    println!("###Name: {}", request_name);
    println!();

    println!("###Directives");
    if directives.is_empty() {
        println!("None");
    } else {
        for directive in directives {
            println!("{}", directive);
        }
    }
    println!();

    println!("###Response");
    println!("HTTP {}", response.status);
    for header in response.headers {
        println!("{}: {}", header.name, header.value);
    }

    println!();
    println!("{}", response.body);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Run,
    ExportCurl,
    ListRequests,
}

struct CliOptions {
    context_json: Option<String>,
    mode: Mode,
}

fn parse_args() -> Result<CliOptions, String> {
    let mut args = env::args().skip(1);
    let mut context = None;
    let mut mode = Mode::Run;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list-requests" => {
                set_mode(&mut mode, Mode::ListRequests)?;
            }
            "--export-curl" => {
                set_mode(&mut mode, Mode::ExportCurl)?;
            }
            "--context-json" => {
                let Some(json) = args.next() else {
                    return Err("Missing value for --context-json".to_string());
                };

                context = Some(json);
            }
            other => {
                return Err(format!("Unknown argument: {}", other));
            }
        }
    }

    Ok(CliOptions {
        context_json: context,
        mode,
    })
}

fn set_mode(current: &mut Mode, next: Mode) -> Result<(), String> {
    if *current != Mode::Run && *current != next {
        return Err("Only one command mode can be used at a time".to_string());
    }

    *current = next;
    Ok(())
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
