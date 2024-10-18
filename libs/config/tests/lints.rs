#![allow(clippy::unwrap_used)]

use std::io::Read;

use hemtt_common::config::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/lints/";

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
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(
            Some(ProjectConfig::test_project()),
            false,
            &hemtt_common::config::PDriveOption::Disallow,
        )
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_config::parse(Some(&ProjectConfig::test_project()), &processed);
    let workspacefiles = WorkspaceFiles::new();
    match parsed {
        Ok(config) => {
            let mut expected = Vec::new();
            std::fs::File::open(folder.join("stdout.ansi"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            let codes = config
                .codes()
                .iter()
                .map(|e| e.diagnostic().unwrap().to_string(&workspacefiles))
                .collect::<Vec<_>>();
            if expected.is_empty() {
                std::fs::write(
                    folder.join("stdout.ansi"),
                    codes.join("\n").replace('\r', "").as_bytes(),
                )
                .unwrap();
            }
            assert_eq!(
                codes.join("\n").replace('\r', ""),
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

bootstrap!(c01_invalid_value);
bootstrap!(c01m_invalid_value_macro);
bootstrap!(c02_duplicate_property_shadow_property);
bootstrap!(c03_duplicate_class);
bootstrap!(c03_duplicate_external);
bootstrap!(c04_missing_parent);
bootstrap!(c05_parent_case);
bootstrap!(c06_unexpected_array);
bootstrap!(c07_expected_array);
bootstrap!(c08_missing_semicolon);
bootstrap!(c09_magwell_missing_magazine);
bootstrap!(c10_class_missing_braces);
