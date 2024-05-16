#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::LayerType;

const ROOT: &str = "tests/preprocessor/";

macro_rules! preprocess {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<preprocess_ $dir>]() {
                preprocess(stringify!($dir));
            }
        }
    };
}

fn preprocess(file: &str) {
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&PathBuf::from(ROOT), LayerType::Source)
        .finish(
            None,
            false,
            &hemtt_common::project::hemtt::PDriveOption::Disallow,
        )
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    std::fs::write(
        format!("tests/preprocessor/{file}.sqfp"),
        processed.as_str(),
    )
    .unwrap();
    let parsed = hemtt_sqf::parser::run(&Database::a3(), &processed).unwrap();
    assert_ne!(parsed.content().len(), 0);
    let mut buffer = Vec::new();
    parsed.compile_to_writer(&processed, &mut buffer).unwrap();
}

preprocess!(gvars);
