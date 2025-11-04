#![allow(clippy::unwrap_used)]

use clap::Parser;

use hemtt::Cli;

#[test]
fn build_simple() {
    let _directory = hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(
        format!("{}/tests/workspace_simple/", env!("CARGO_MANIFEST_DIR")),
    ));
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "dev"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "build"])).unwrap();
}

#[test]
fn build_post_release() {
    let _directory =
        hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(format!(
            "{}/tests/workspace_post_release/",
            env!("CARGO_MANIFEST_DIR")
        )));
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "release"])).unwrap();
}
