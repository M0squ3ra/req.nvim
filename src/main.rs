mod http;
mod req;

use req::model::{HttpMethod, Request};

fn main() {
    let request = Request {
        method: HttpMethod::Get,
        url: "https://example.com".to_string(),
        headers: vec![],
        body: None,
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
