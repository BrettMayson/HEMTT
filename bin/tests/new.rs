#![allow(clippy::unwrap_used)]

use clap::Parser;
use hemtt::Cli;

#[test]
#[serial_test::serial]
fn new() {
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "new", "test", "--in-test"])).unwrap();
}
