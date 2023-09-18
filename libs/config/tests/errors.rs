use std::io::Read;

use hemtt_preprocessor::Processor;

const ROOT: &str = "tests/errors/";

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
            let errors = config
                .errors()
                .iter()
                .map(|e| e.generate_processed_report(&processed).unwrap())
                .collect::<Vec<_>>();
            if expected.is_empty() {
                std::fs::write(
                    folder.join("stdout.ansi"),
                    errors.join("\n").replace('\r', "").as_bytes(),
                )
                .unwrap();
            }
            assert_eq!(
                errors.join("\n").replace('\r', "").replace(r"\u{1b}[38;5;201m─\u{1b}[0m\u{1b}[38;5;201m┬\u{1b}[0m  \n \u{1b}[38;5;240m  │\u{1b}[0m ", r"\u{1b}[38;5;201m┬\u{1b}[0m  \n \u{1b}[38;5;240m  │\u{1b}[0m"),
                String::from_utf8(expected).unwrap().replace('\r', "")
            );
        }
        // Errors may occur, but they should be handled, if one is not a handler should be created
        Err(e) => {
            panic!("{:#?}", e)
        }
    }
}

bootstrap!(simple);
bootstrap!(arrays);
