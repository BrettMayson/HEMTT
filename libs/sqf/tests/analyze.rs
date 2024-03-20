#![allow(clippy::unwrap_used)]

use std::io::Read;

use hemtt_preprocessor::Processor;
use hemtt_sqf::{analyze::analyze, parser::database::Database};
use hemtt_workspace::{reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/analyze/";

macro_rules! analyze {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                test_analyze(stringify!($dir));
            }
        }
    };
}

fn test_analyze(dir: &str) {
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
    let database = Database::default();
    let workspace_files = WorkspaceFiles::new();
    match hemtt_sqf::parser::run(&database, &processed) {
        Ok(sqf) => {
            let (warnings, _errors) = analyze(&sqf, None, &processed, None, &database);
            let stdout = warnings
                .iter()
                .map(|e| e.diagnostic().unwrap().to_string(&workspace_files))
                .collect::<Vec<_>>()
                .join("\n")
                .replace('\r', "");
            let mut expected = Vec::new();
            std::fs::File::open(folder.join("stdout.ansi"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            if expected.is_empty() {
                std::fs::write(folder.join("stdout.ansi"), stdout.as_bytes()).unwrap();
            }
            let expected = String::from_utf8_lossy(&expected).replace('\r', "");
            assert_eq!(stdout, expected);
        }
        Err(hemtt_sqf::parser::ParserError::ParsingError(e)) => {
            for error in e {
                println!(
                    "{}",
                    error.diagnostic().unwrap().to_string(&workspace_files)
                );
            }
            panic!("failed to parse");
        }
        Err(e) => panic!("{e:?}"),
    };
}

analyze!(saa1_if_assign);
analyze!(saa2_find_in_str);
analyze!(saa3_typename);
analyze!(saa4_str_format);
// analyze!(saa5_select_parse_number);
