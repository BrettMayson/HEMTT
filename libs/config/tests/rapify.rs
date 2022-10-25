use std::io::Read;

use hemtt_config::{Config, Parse, Rapify};
use hemtt_preprocessor::{preprocess_file, resolvers::LocalResolver};

const ROOT: &str = "tests/rapify/";

#[test]
fn rapify() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            let mut resolver = LocalResolver::new();
            println!(
                "rapify `{}`",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let tokens = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &mut resolver,
            )
            .unwrap();
            let rapified = Config::parse(&mut tokens.into_iter().peekable()).unwrap();
            let mut output = Vec::new();
            rapified.rapify(&mut output, 0).unwrap();
            // read binary file
            let mut expected = Vec::new();
            std::fs::File::open(file.path().join("expected.bin"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            assert_eq!(output, expected);
        }
    }
}
