use std::{io::Read, path::PathBuf};

use hemtt_preprocessor::{preprocess_file, Resolver};
use vfs::PhysicalFS;

const ROOT: &str = "tests/errors/";

#[test]
fn errors() {
    // The output is slightly different on non-windows platforms
    if !cfg!(windows) {
        return;
    }
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
            let parsed = hemtt_config::parse(&processed);
            match parsed {
                Ok(config) => {
                    let mut expected = Vec::new();
                    std::fs::File::open(file.path().join("stdout.ansi"))
                        .unwrap()
                        .read_to_end(&mut expected)
                        .unwrap();
                    let errors = config
                        .errors()
                        .iter()
                        .map(|e| e.generate_processed_report(&processed).unwrap())
                        .collect::<Vec<_>>();
                    if expected.is_empty() {
                        std::fs::write(
                            file.path().join("stdout.ansi"),
                            errors.join("\n").replace('\r', "").as_bytes(),
                        )
                        .unwrap();
                    }
                    assert_eq!(
                        errors.join("\n").replace('\r', ""),
                        String::from_utf8(expected).unwrap().replace('\r', "")
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
