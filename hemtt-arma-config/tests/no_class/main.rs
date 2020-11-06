#[test]
fn string() {
    hemtt_arma_config::tokenize(r#"something = "a string""#).unwrap();
}

#[test]
fn config() {
    hemtt_arma_config::tokenize(&std::fs::read_to_string("tests/no_class/config.cpp").unwrap())
        .unwrap();
}
