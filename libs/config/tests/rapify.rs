use std::io::Read;

use hemtt_config::{Config, Parse, Rapify};
use hemtt_preprocessor::{preprocess_file, resolvers::LocalResolver};
use peekmore::PeekMore;

const ROOT: &str = "tests/rapify/";

#[test]
fn rapify() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            println!(
                "rapify `{}`",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let tokens = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &LocalResolver::new(),
            )
            .unwrap();
            let rapified = Config::parse(
                &hemtt_config::Options::default(),
                &mut tokens.into_iter().peekmore(),
                &hemtt_tokens::Token::builtin(None),
            )
            .unwrap();
            let mut output = Vec::new();
            rapified.rapify(&mut output, 0).unwrap();
            let mut expected = Vec::new();
            std::fs::File::open(file.path().join("expected.bin"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            assert_eq!(output, expected);
        }
    }
}
