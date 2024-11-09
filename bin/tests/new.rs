#![allow(clippy::unwrap_used)]

use clap::Parser;
use hemtt::Cli;

use sealed_test::prelude::*;

#[sealed_test]
fn new() {
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "new", "test", "--in-test"])).unwrap();
}
