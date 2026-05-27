use super::{
    args::OutputFormat,
    commands::CommandOutput,
    dto::{ExecutedResponseOutput, RequestListOutput},
};

pub fn render(output: CommandOutput, format: OutputFormat) -> Result<(), String> {
    match format {
        OutputFormat::Text => render_text(output),
        OutputFormat::Json => render_json(output),
    }
}

fn render_text(output: CommandOutput) -> Result<(), String> {
    match output {
        CommandOutput::RequestList(output) => {
            print_request_list(output);
        }
        CommandOutput::Check(_) => {
            println!("OK");
        }
        CommandOutput::Curl(output) => {
            println!("{}", output.command);
        }
        CommandOutput::ExecutedResponse(output) => {
            print_response(output);
        }
    }

    Ok(())
}

fn render_json(output: CommandOutput) -> Result<(), String> {
    let json = match output {
        CommandOutput::RequestList(output) => serde_json::to_string(&output),
        CommandOutput::Check(output) => serde_json::to_string(&output),
        CommandOutput::Curl(output) => serde_json::to_string(&output),
        CommandOutput::ExecutedResponse(output) => serde_json::to_string(&output),
    }
    .map_err(|error| format!("Output JSON error: {error}"))?;

    println!("{}", json);
    Ok(())
}

fn print_request_list(output: RequestListOutput) {
    if output.requests.is_empty() {
        println!("No requests found");
        return;
    }

    for request in output.requests {
        let name = request
            .name
            .unwrap_or_else(|| "Untitled request".to_string());
        println!("{} {}:{}", name, request.start_line, request.end_line);
    }
}

fn print_response(output: ExecutedResponseOutput) {
    println!("###Name: {}", output.request.name);
    println!();

    println!("###Directives");
    if output.request.directives.is_empty() {
        println!("None");
    } else {
        for directive in output.request.directives {
            println!("{}", directive);
        }
    }
    println!();

    println!("###Response");
    println!("HTTP {}", output.response.status);
    for header in output.response.headers {
        println!("{}: {}", header.name, header.value);
    }

    println!();
    println!("{}", output.response.body);
}
