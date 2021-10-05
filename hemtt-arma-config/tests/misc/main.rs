#[test]
fn ignore_pound() {
    let config = hemtt_arma_config::preprocess(
        hemtt_arma_config::tokenize(
            &std::fs::read_to_string("tests/misc/files/ignore_pound.in.hpp").unwrap(),
            "tests/misc/files/ignore_pound.in.hpp",
        )
        .unwrap(),
        ".",
        hemtt_arma_config::resolver::Basic,
    );
    let config = hemtt_arma_config::render(config.unwrap());
    assert_eq!(
        std::fs::read_to_string("tests/misc/files/ignore_pound.out.hpp")
            .unwrap()
            .replace('\r', ""),
        config.export()
    );
}
