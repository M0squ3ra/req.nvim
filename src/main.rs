use std::io::{self, Read};

mod http;
mod req;

fn main() {
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
        .collect::<Vec<_>>();

    let response = match http::client::execute(parsed_request.request) {
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
