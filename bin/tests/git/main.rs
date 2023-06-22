use hemtt::cli;

#[test]
fn theseus() {
    let _ = std::fs::remove_dir_all("tests/git/theseus");
    git2::Repository::clone(
        "https://github.com/theseus-aegis/mods",
        "./tests/git/theseus",
    )
    .unwrap();

    std::env::set_current_dir("tests/git/theseus").unwrap();
    hemtt::execute(&cli().get_matches_from(vec!["hemtt", "build", "--in-test"])).unwrap();
}
