use std::io::{self, Read};

mod http;
mod req;

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");
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

    match http::client::execute(request) {
        Ok(response) => {
            println!("HTTP {}", response.status);
            for header in response.headers {
                println!("{}: {}", header.name, header.value);
            }

            println!();
            println!("{}", response.body);
        }
        Err(error) => {
            eprintln!("request error: {}", error);
            std::process::exit(1);
        }
    }
}
