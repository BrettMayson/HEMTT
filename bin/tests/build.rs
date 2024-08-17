#![allow(clippy::unwrap_used)]

use sealed_test::prelude::*;

use hemtt::cli;

#[sealed_test]
fn build_alpha() {
    std::env::set_current_dir(format!("{}/tests/alpha", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "dev", "--in-test"])).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "build", "--in-test"])).unwrap();
}

#[sealed_test]
fn build_bravo() {
    std::env::set_current_dir(format!("{}/tests/bravo", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "script", "test"])).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "release", "--in-test"])).unwrap();
}

#[sealed_test]
fn build_bravo_sqfc() {
    std::env::set_current_dir(format!("{}/tests/bravo", env!("CARGO_MANIFEST_DIR"))).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "release", "--expsqfc"])).unwrap();
}
