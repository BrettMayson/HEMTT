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
            let vfs =
                PhysicalFS::new(PathBuf::from(ROOT).join(file.path().file_name().unwrap())).into();
            let resolver = Resolver::new(&vfs, Default::default());
            for dir in vfs.read_dir().unwrap() {
                println!("  {:?}", dir.as_str());
            }
            let processed = preprocess_file(&vfs.join("source.hpp").unwrap(), &resolver).unwrap();
            std::fs::write(file.path().join("generated.hpp"), processed.output()).unwrap();
            assert_eq!(processed.output(), expected.replace('\r', ""));
        }
    }
}
