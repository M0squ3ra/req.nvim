use std::io::{self, Read};

mod http;
mod req;

use req::parser::ast::{Directive, ReqLine};

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");
    let document = req::parser::parse_document(&input);
    let request = match req::parser::parse_request(&input) {
        Ok(request) => request,
        Err(error) => {
            eprintln!(
                "Parse error at {}:{}: {}",
                error.line, error.column, error.message
            );
            std::process::exit(1);
        }
    };

    let request_name = request
        .name
        .clone()
        .unwrap_or_else(|| "Untitled request".to_string());
    let directives = document
        .requests
        .first()
        .map(|block| {
            block
                .lines
                .iter()
                .filter_map(|line| match line {
                    ReqLine::Directive(Directive::Env(env)) => Some(format!("@env {}", env)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

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
