#![allow(clippy::unwrap_used)]

use hemtt::cli;

use sealed_test::prelude::*;

#[sealed_test]
fn new() {
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "new", "test", "--in-test"])).unwrap();
}
