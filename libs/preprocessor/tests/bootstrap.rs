use hemtt_preprocessor::{preprocess_file, resolvers::LocalResolver, Processed};

const ROOT: &str = "tests/bootstrap/";

#[test]
fn bootstrap() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            let expected = std::fs::read_to_string(file.path().join("expected.hpp")).unwrap();
            let mut resolver = LocalResolver::new();
            println!(
                "bootstrap `{}`",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let tokens = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &mut resolver,
            )
            .unwrap();
            let processed = Processed::from(tokens);
            let map = processed.get_source_map(file.path().join("expected.hpp"));
            std::fs::write(file.path().join("expected.hpp.map"), map).unwrap();
            assert_eq!(processed.output(), expected);
        }
    }
}
