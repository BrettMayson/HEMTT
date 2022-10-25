use hemtt::cli;

#[test]
pub fn dev() {
    std::env::set_current_dir("tests/alpha").unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "dev", "-b"])).unwrap();
}
