use std::path::PathBuf;

fn resolver(name: &str) -> PathBuf {
    PathBuf::from(format!("tests/preprocess/{}", name))
}

#[test]
fn basic_if() {
    let config = hemtt_arma_config::preprocess(
        hemtt_arma_config::tokenize(
            &std::fs::read_to_string("tests/if/files/basic_if.in.hpp").unwrap(),
        )
        .unwrap(),
        &resolver,
    );
    let config = hemtt_arma_config::render(config.unwrap());
    assert_eq!(
        std::fs::read_to_string("tests/if/files/basic_if.out.hpp")
            .unwrap()
            .replace('\r', ""),
        config
    );
}

#[test]
fn nested_if() {
    let config = hemtt_arma_config::preprocess(
        hemtt_arma_config::tokenize(
            &std::fs::read_to_string("tests/if/files/nested_if.in.hpp").unwrap(),
        )
        .unwrap(),
        &resolver,
    );
    let config = hemtt_arma_config::render(config.unwrap());
    assert_eq!(
        std::fs::read_to_string("tests/if/files/nested_if.out.hpp")
            .unwrap()
            .replace('\r', ""),
        config
    );
}
