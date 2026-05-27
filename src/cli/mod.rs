use std::io::{self, Read};

mod args;
mod commands;
mod dto;
mod output;

pub fn run() -> Result<(), String> {
    let options = args::parse_args()?;
    let input = read_stdin()?;
    let output = commands::run_command(&options, &input)?;

    output::render(output, options.output_format)
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("Failed to read stdin: {error}"))?;

    Ok(input)
}
