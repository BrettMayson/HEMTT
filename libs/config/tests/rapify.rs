use std::{io::Read, path::PathBuf};

use chumsky::Parser;
use hemtt_config::{parse::config, rapify::Rapify};
use hemtt_preprocessor::{preprocess_file, Resolver};
use vfs::PhysicalFS;

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
            let vfs = PhysicalFS::new(PathBuf::from(ROOT).join(file.path())).into();
            let resolver = Resolver::new(&vfs, Default::default());
            let processed = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &resolver,
            )
            .unwrap();
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
