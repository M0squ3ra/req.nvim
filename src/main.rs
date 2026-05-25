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
    let parsed_request = match req_core::req::parser::parse_request(&input) {
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

    let request = match req_core::req::resolver::resolve_request(parsed_request, options.context) {
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
    context: req_core::req::resolver::ResolveContext,
    export_curl: bool,
}

fn parse_args() -> Result<CliOptions, String> {
    let mut args = env::args().skip(1);
    let mut context = None;
    let mut export_curl = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--export-curl" => {
                export_curl = true;
            }
            "--context-json" => {
                let Some(json) = args.next() else {
                    return Err("Missing value for --context-json".to_string());
                };

                context = Some(
                    req_core::req::resolver::ResolveContext::from_json(&json)
                        .map_err(|error| error.message)?,
                );
            }
            other => {
                return Err(format!("Unknown argument: {}", other));
            }
        }
    }

    Ok(CliOptions {
        context: context.unwrap_or_default(),
        export_curl,
    })
}
