#[test]
fn cba_common_macros() {
    let config = hemtt_arma_config::preprocess(
        hemtt_arma_config::tokenize(
            &std::fs::read_to_string("tests/cba/config.cpp").unwrap(),
            "tests/cba/config.cpp",
        )
        .unwrap(),
        ".",
        hemtt_arma_config::resolver::Basic,
    )
    .unwrap();
    let simplified = hemtt_arma_config::simplify::Config::from_ast(
        hemtt_arma_config::parse(&hemtt_arma_config::render(config).export(), "test").unwrap(),
    )
    .unwrap();
    let mut buf = Vec::new();
    simplified.write_rapified(&mut buf).unwrap();
}
