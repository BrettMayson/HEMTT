#![allow(clippy::unwrap_used)]

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::{LayerType, reporting::WorkspaceFiles};

macro_rules! errors {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<errors_ $dir>]() {
                insta::assert_snapshot!(errors(stringify!($dir)));
            }
        }
    };
}

errors!(spe1_invalid_token);
errors!(spe2_unparseable);

const ROOT: &str = "tests/errors/";

fn errors(file: &str) -> String {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_sqf::parser::run(&Database::a3(false), &processed).unwrap_err();
    let codes = parsed.codes();
    codes
        .iter()
        .map(|e| e.diagnostic().unwrap().to_string(&WorkspaceFiles::new()))
        .collect::<Vec<_>>()
        .join("\n")
        .replace('\r', "")
}
