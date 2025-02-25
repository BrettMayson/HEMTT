#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use hemtt_preprocessor::Processor;
use hemtt_sqf::{parser::database::Database, Statement};
use hemtt_workspace::{reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/simple/";

macro_rules! simple {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                let ast = simple(stringify!($dir));
                insta::assert_debug_snapshot!(ast);
            }
        }
    };
}

simple!(dev);
simple!(eventhandler);
simple!(foreach);
simple!(format_font);
simple!(get_visibility);
simple!(hash_select);
simple!(hello);
simple!(include);
simple!(oneline);
simple!(semicolons);

fn simple(file: &str) -> Vec<Statement> {
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&PathBuf::from(ROOT), LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    std::fs::write(format!("tests/simple/{file}.sqfp"), processed.as_str()).unwrap();
    let parsed = match hemtt_sqf::parser::run(&Database::a3(false), &processed) {
        Ok(sqf) => sqf,
        Err(hemtt_sqf::parser::ParserError::ParsingError(e)) => {
            for error in e {
                println!(
                    "{}",
                    error
                        .diagnostic()
                        .unwrap()
                        .to_string(&WorkspaceFiles::new())
                );
            }
            panic!("failed to parse");
        }
        Err(e) => panic!("{e:?}"),
    };
    assert_ne!(parsed.content().len(), 0);
    std::fs::write(
        format!("tests/simple/{file}.sqfast"),
        format!("{:#?}", parsed.content()),
    )
    .unwrap();
    parsed.content().to_vec()
}
