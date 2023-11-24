use std::path::PathBuf;

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;

const ROOT: &str = "tests/simple/";

macro_rules! simple {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                simple(stringify!($dir));
            }
        }
    };
}

fn simple(file: &str) {
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&PathBuf::from(ROOT))
        .finish(None)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_sqf::parser::run(&Database::default(), &processed).unwrap();
    assert_ne!(parsed.content.len(), 0);
    let mut buffer = Vec::new();
    parsed.compile_to_writer(&processed, &mut buffer).unwrap();
}

simple!(hello);
simple!(get_visibility);
