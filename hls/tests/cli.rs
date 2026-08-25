//! The server is launched by the editor extension with a port to connect to.
//! Anything else must fail with a message, not a panic - see #1281, where a
//! shell completion setup ran it bare and got a backtrace on every terminal.

use std::process::Command;

fn run(args: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_hemtt-language-server"))
        .args(args)
        .output()
        .expect("failed to run the language server");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn regression_1281_no_port_prints_usage() {
    let (code, stderr) = run(&[]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stderr.contains("usage:"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn regression_1281_invalid_port_does_not_panic() {
    let (code, stderr) = run(&["not-a-port"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("failed to connect"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn unreachable_port_does_not_panic() {
    // port 0 is reserved, so nothing can be listening on it
    let (code, stderr) = run(&["0"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}
