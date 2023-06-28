use hemtt_preprocessor::{preprocess_file, LocalResolver, Processed};

const ROOT: &str = "tests/bootstrap/";

#[test]
fn bootstrap() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            let expected = std::fs::read_to_string(file.path().join("expected.hpp")).unwrap();
            let resolver = LocalResolver::default();
            println!(
                "bootstrap {:?}",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let tokens = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &resolver,
            )
            .unwrap();
            let processed = Processed::from_tokens(&resolver, tokens);
            std::fs::write(file.path().join("generated.hpp"), processed.output()).unwrap();
            assert_eq!(processed.output(), expected.replace('\r', ""));
        }
    }
}
