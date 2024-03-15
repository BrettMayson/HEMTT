use std::io::Read;

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::{reporting::WorkspaceFiles, LayerType};

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
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(
            None,
            false,
            &hemtt_common::project::hemtt::PDriveOption::Disallow,
        )
        .unwrap();
    let source = workspace.join("source.sqf").unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_sqf::parser::run(&Database::default(), &processed).unwrap_err();
    let codes = parsed.codes();
    let mut expected = Vec::new();
    std::fs::File::open(folder.join("stderr.ansi"))
        .unwrap()
        .read_to_end(&mut expected)
        .unwrap();
    let errors = codes
        .iter()
        .map(|e| e.diagnostic().unwrap().to_string(&WorkspaceFiles::new()))
        .collect::<Vec<_>>()
        .join("\n")
        .replace('\r', "");
    if expected.is_empty() {
        std::fs::write(folder.join("stderr.ansi"), errors.as_bytes()).unwrap();
    }
    let expected = String::from_utf8_lossy(&expected).replace('\r', "");
    assert_eq!(errors, expected);
}

errors!(spe1_invalid_token);
errors!(spe2_unparseable);
