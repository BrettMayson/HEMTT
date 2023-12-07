use std::io::Read;

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::{database::Database, ParserError};

const ROOT: &str = "tests/errors/";

macro_rules! errors {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<errors_ $dir>]() {
                errors(stringify!($dir));
            }
        }
    };
}

fn errors(dir: &str) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&folder)
        .finish(None)
        .unwrap();
    let source = workspace.join("source.sqf").unwrap();
    let processed = Processor::run(&source).unwrap();
    let ParserError::ParsingError(parsed) =
        hemtt_sqf::parser::run(&Database::default(), &processed).unwrap_err()
    else {
        panic!("Expected parsing error");
    };
    let mut expected = Vec::new();
    std::fs::File::open(folder.join("error.ansi"))
        .unwrap()
        .read_to_end(&mut expected)
        .unwrap();
    let errors = parsed
        .iter()
        .map(|e| e.report_generate_processed(&processed).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    if expected.is_empty() {
        std::fs::write(folder.join("error.ansi"), errors.as_bytes()).unwrap();
    }
    assert_eq!(errors.as_bytes(), expected);
}

errors!(spe1_unparseable);
