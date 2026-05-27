use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Run,
    Check,
    ExportCurl,
    ListRequests,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub struct CliOptions {
    pub context_json: Option<String>,
    pub mode: Mode,
    pub output_format: OutputFormat,
}

pub fn parse_args() -> Result<CliOptions, String> {
    let mut args = env::args().skip(1);
    let mut context = None;
    let mut mode = Mode::Run;
    let mut output_format = OutputFormat::Text;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list-requests" => {
                set_mode(&mut mode, Mode::ListRequests)?;
            }
            "--check" => {
                set_mode(&mut mode, Mode::Check)?;
            }
            "--export-curl" => {
                set_mode(&mut mode, Mode::ExportCurl)?;
            }
            "--output-json" => {
                output_format = OutputFormat::Json;
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
        output_format,
    })
}

fn set_mode(current: &mut Mode, next: Mode) -> Result<(), String> {
    if *current != Mode::Run && *current != next {
        return Err("Only one command mode can be used at a time".to_string());
    }

    *current = next;
    Ok(())
}
