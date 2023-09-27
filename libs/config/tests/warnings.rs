use std::io::Read;

use hemtt_preprocessor::Processor;

const ROOT: &str = "tests/warnings/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<bootstrap_ $dir>]() {
                check(stringify!($dir));
            }
        }
    };
}

fn check(dir: &str) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&folder)
        .finish()
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_config::parse(&processed);
    match parsed {
        Ok(config) => {
            let mut expected = Vec::new();
            std::fs::File::open(folder.join("stdout.ansi"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            let warnings = config
                .warnings()
                .iter()
                .map(|e| e.generate_processed_report(&processed).unwrap())
                .collect::<Vec<_>>();
            if expected.is_empty() {
                std::fs::write(
                    folder.join("stdout.ansi"),
                    warnings.join("\n").replace('\r', "").as_bytes(),
                )
                .unwrap();
            }
            assert_eq!(
                warnings.join("\n").replace('\r', ""),
                String::from_utf8(expected).unwrap().replace('\r', "")
            );
        }
        // warnings may occur, but they should be handled, if one is not a handler should be created
        Err(e) => {
            panic!("{:#?}", e)
        }
    }
}

bootstrap!(cw1_parent_case);
