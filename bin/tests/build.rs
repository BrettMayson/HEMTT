#![allow(clippy::unwrap_used)]

use clap::Parser;
use sealed_test::prelude::*;

use hemtt::Cli;

#[sealed_test]
fn build_alpha() {
    std::env::set_current_dir(format!("{}/tests/alpha", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "dev", "--in-test"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "build", "--in-test"])).unwrap();
}

#[sealed_test]
fn build_bravo() {
    std::env::set_current_dir(format!("{}/tests/bravo", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "script", "test"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "release", "--in-test"])).unwrap();
}
