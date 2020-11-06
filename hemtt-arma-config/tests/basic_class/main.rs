#[test]
fn config() {
    println!(
        "{:#?}",
        hemtt_arma_config::tokenize(
            &std::fs::read_to_string("tests/basic_class/config.cpp").unwrap()
        )
        .unwrap()
    );
}
