#![allow(clippy::unwrap_used)]

use std::io::Read;

use hemtt_common::project::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/warnings/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<config_warning_ $dir>]() {
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
            None,
            false,
            &hemtt_common::project::hemtt::PDriveOption::Disallow,
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
            let warnings = config
                .warnings()
                .iter()
                .map(|e| e.diagnostic().unwrap().to_string(&workspacefiles))
                .collect::<Vec<_>>();
            if expected.is_empty() {
                std::fs::write(
                    folder.join("stdout.ansi"),
                    warnings.join("\n").replace('\r', "").as_bytes(),
                )
                .unwrap();
            }
            assert_eq!(
                warnings.join("\n").replace('\r', ""),
                String::from_utf8(expected).unwrap().replace('\r', "")
            );
        }
        // warnings may occur, but they should be handled, if one is not a handler should be created
        Err(e) => {
            for e in &e {
                eprintln!("{}", e.diagnostic().unwrap().to_string(&workspacefiles));
            }
            panic!("Error parsing config");
        }
    }
}

bootstrap!(cw1_parent_case);
bootstrap!(cw2_magwell_missing_magazine);
