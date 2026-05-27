use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_req(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_req-nvim"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();

    child.wait_with_output().unwrap()
}

#[test]
fn check_resolves_request_without_executing_it() {
    let output = run_req(
        &[
            "--check",
            "--context-json",
            r#"{"envs":{"dev":{"BASE_URL":"http://127.0.0.1:1"}}}"#,
        ],
        "### Local\n# @env dev\n\nGET {{BASE_URL}}/health\n",
    );

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "OK\n");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[test]
fn check_can_render_json() {
    let output = run_req(
        &["--check", "--output-json"],
        "GET https://example.com/health\n",
    );

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "{\"ok\":true}\n");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[test]
fn check_reports_resolve_errors() {
    let output = run_req(&["--check"], "GET {{BASE_URL}}/health\n");

    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "Resolve error: Missing variable: BASE_URL\n"
    );
}

#[test]
fn list_requests_renders_text_by_default() {
    let output = run_req(
        &["--list-requests"],
        "### Health\nGET https://example.com/health\n\n### Create\nPOST https://example.com/users\n",
    );

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Health 1:3\nCreate 4:5\n"
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

#[test]
fn list_requests_can_render_json() {
    let output = run_req(
        &["--list-requests", "--output-json"],
        "### Health\nGET https://example.com/health\n",
    );

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "{\"requests\":[{\"name\":\"Health\",\"start_line\":1,\"end_line\":2}]}\n"
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}
