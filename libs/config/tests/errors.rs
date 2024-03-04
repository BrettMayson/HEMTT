use std::io::Read;

use hemtt_common::{reporting::WorkspaceFiles, workspace::LayerType};
use hemtt_preprocessor::Processor;

const ROOT: &str = "tests/errors/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<config_error_ $dir>]() {
                check(stringify!($dir));
            }
        }
    };
}

fn check(dir: &str) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false)
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_config::parse(None, &processed);
    let workspacefiles = WorkspaceFiles::new();
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
                .map(|e| e.diagnostic().unwrap().to_string(&workspacefiles))
                .collect::<Vec<_>>();
            if expected.is_empty() {
                std::fs::write(
                    folder.join("stdout.ansi"),
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
            for e in &e {
                eprintln!("{}", e.diagnostic().unwrap().to_string(&workspacefiles));
            }
            panic!("Error parsing config");
        }
    }
}

bootstrap!(ce1_invalid_value);
bootstrap!(ce2_invalid_value_macro);
bootstrap!(ce3_duplicate_property_separate);
bootstrap!(ce3_duplicate_property_shadow_property);
bootstrap!(ce4_missing_semicolon);
bootstrap!(ce5_unexpected_array);
bootstrap!(ce6_expected_array);
bootstrap!(ce7_missing_parent);
bootstrap!(ce7_missing_parent_tree);
