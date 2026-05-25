use std::{
    env,
    io::{self, Read},
};

mod http;
mod req;

fn main() {
    let context = match parse_context_arg() {
        Ok(context) => context,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");
    let parsed_request = match req::parser::parse_request(&input) {
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

    let request = match req::resolver::resolve_request(parsed_request, context) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("Resolve error: {}", error.message);
            std::process::exit(1);
        }
    };

    let response = match http::client::execute(request) {
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

fn parse_context_arg() -> Result<req::resolver::ResolveContext, String> {
    let mut args = env::args().skip(1);
    let mut context = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--context-json" => {
                let Some(json) = args.next() else {
                    return Err("Missing value for --context-json".to_string());
                };

                context = Some(
                    req::resolver::ResolveContext::from_json(&json)
                        .map_err(|error| error.message)?,
                );
            }
            other => {
                return Err(format!("Unknown argument: {}", other));
            }
        }
    }

    Ok(context.unwrap_or_default())
}
