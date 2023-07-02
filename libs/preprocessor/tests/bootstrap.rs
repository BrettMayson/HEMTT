use std::path::PathBuf;

use hemtt_preprocessor::{preprocess_file, Resolver};
use vfs::PhysicalFS;

const ROOT: &str = "tests/bootstrap/";

#[test]
fn bootstrap() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            let expected = std::fs::read_to_string(file.path().join("expected.hpp")).unwrap();
            let vfs = PhysicalFS::new(PathBuf::from(ROOT).join(file.path())).into();
            let resolver = Resolver::new(&vfs, Default::default());
            println!(
                "bootstrap {:?}",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let processed = preprocess_file(
                &file.path().join("source.hpp").display().to_string(),
                &resolver,
            )
            .unwrap();
            std::fs::write(file.path().join("generated.hpp"), processed.output()).unwrap();
            assert_eq!(processed.output(), expected.replace('\r', ""));
        }
    }
}
