#![allow(clippy::unwrap_used)]

use clap::Parser;
use hemtt::Cli;

#[test]
fn new() {
    let _directory = hemtt_test::directory::TemporaryDirectory::new();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "new", "test"])).unwrap();
}
