use std::io::Read;

use chumsky::Parser;
use hemtt_config::{parse::config, rapify::Rapify};
use hemtt_preprocessor::{preprocess_file, LocalResolver, Processed};

const ROOT: &str = "tests/rapify/";

#[test]
fn rapify() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            println!(
                "rapify {:?}",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let resolver = LocalResolver::default();
            let tokens = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &resolver,
            )
            .unwrap();
            let processed = Processed::from_tokens(&resolver, tokens);
            let rapified = config().parse(processed.output()).unwrap();
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
