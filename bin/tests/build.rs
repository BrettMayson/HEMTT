#![allow(clippy::unwrap_used)]

use clap::Parser;

use hemtt::Cli;

#[test]
#[serial_test::serial]
fn build_alpha() {
    std::env::set_current_dir(format!("{}/tests/alpha", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "dev", "--in-test"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "build", "--in-test"])).unwrap();
}

#[test]
#[serial_test::serial]
fn build_bravo() {
    std::env::set_current_dir(format!("{}/tests/bravo", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "script", "test"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "release", "--in-test"])).unwrap();
}
