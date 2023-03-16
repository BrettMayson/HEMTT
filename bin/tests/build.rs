use hemtt::cli;

#[test]
pub fn build() {
    std::env::set_current_dir("tests/alpha").unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "build", "--in-test"])).unwrap();

    std::env::set_current_dir("../bravo").unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "release", "--in-test"])).unwrap();
}
