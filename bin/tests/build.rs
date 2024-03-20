#![allow(clippy::unwrap_used)]

use hemtt::cli;

#[test]
/// # Panics
/// Will panic if there is an issue with the test
pub fn build() {
    std::env::set_current_dir("tests/alpha").unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "dev", "--in-test"])).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "build", "--in-test"])).unwrap();

    std::env::set_current_dir("../bravo").unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "script", "test"])).unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "release", "--in-test"])).unwrap();
}
