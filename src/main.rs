use std::{
    env,
    io::{self, Read},
};

fn main() {
    let options = match parse_args() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");

    if options.list_requests {
        match serde_json::to_string(&req_core::req::listing::list_requests(&input)) {
            Ok(json) => println!("{}", json),
            Err(error) => {
                eprintln!("List requests error: {}", error);
                std::process::exit(1);
            }
        }

        return;
    }

    let document = req_core::req::parser::parse(&input);
    let parsed_request = match req_core::req::lowering::lower_first_request(&document) {
        Ok(parsed_request) => parsed_request,
        Err(error) => {
            eprintln!(
                "Parse error at {}:{}: {}",
                error.line, error.column, error.message
            );
            std::process::exit(1);
        }
    };

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

    let context = match context_for_request(&parsed_request, options.context_json.as_deref()) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("Resolve error: {}", error.message);
            std::process::exit(1);
        }
    };

    let request = match req_core::req::resolver::resolve_request(parsed_request, context) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("Resolve error: {}", error.message);
            std::process::exit(1);
        }
    };

    if options.export_curl {
        println!("{}", req_core::req::curl::to_curl(&request));
        return;
    }

    let response = match req_core::http::client::execute(request) {
        Ok(response) => response,
        Err(error) => {
            eprintln!("request error: {}", error);
            std::process::exit(1);
        }
    };

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

struct CliOptions {
    context_json: Option<String>,
    export_curl: bool,
    list_requests: bool,
}

fn parse_args() -> Result<CliOptions, String> {
    let mut args = env::args().skip(1);
    let mut context = None;
    let mut export_curl = false;
    let mut list_requests = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list-requests" => {
                list_requests = true;
            }
            "--export-curl" => {
                export_curl = true;
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
        export_curl,
        list_requests,
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
