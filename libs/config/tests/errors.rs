use std::{io::Read, path::PathBuf};

use hemtt_preprocessor::{preprocess_file, Resolver};
use vfs::PhysicalFS;

const ROOT: &str = "tests/errors/";

#[test]
fn errors() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();
        if file.path().is_dir() {
            println!(
                "errors {:?}",
                file.path().file_name().unwrap().to_str().unwrap()
            );
            let vfs =
                PhysicalFS::new(PathBuf::from(ROOT).join(file.path().file_name().unwrap())).into();
            let resolver = Resolver::new(&vfs, Default::default());
            let processed = preprocess_file(&vfs.join("source.hpp").unwrap(), &resolver).unwrap();
            let rapified = hemtt_config::parse(&processed);
            match rapified {
                Ok(config) => {
                    let mut expected = Vec::new();
                    std::fs::File::open(file.path().join("stdout.ansi"))
                        .unwrap()
                        .read_to_end(&mut expected)
                        .unwrap();
                    // if expected.is_empty() {
                    // write
                    std::fs::write(
                        file.path().join("stdout.ansi"),
                        config.errors().join("\n").replace('\r', "").as_bytes(),
                    )
                    .unwrap();
                    // }
                    assert_eq!(
                        config.errors().join("\n"),
                        String::from_utf8(expected).unwrap()
                    );
                }
                // Errors may occur, but they should be handled, if one is not a handler should be created
                Err(e) => {
                    panic!("{:#?}", e)
                }
            }
        }
    }
}
