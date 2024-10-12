#![allow(clippy::unwrap_used)]

use std::{io::Read, sync::Arc};

use hemtt_common::config::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{analyze::analyze, parser::database::Database};
use hemtt_workspace::{addons::Addon, reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/lints/";

macro_rules! analyze {
    ($dir:ident, $ignore:expr) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                test_analyze(stringify!($dir), $ignore);
            }
        }
    };
}

fn test_analyze(dir: &str, ignore_inspector: bool) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("source.sqf").unwrap();
    let processed = Processor::run(&source).unwrap();
    let database = Arc::new(Database::a3(false));
    let workspace_files = WorkspaceFiles::new();

    let config_path_full = std::path::PathBuf::from(ROOT).join("project_tests.toml");
    let config = ProjectConfig::from_file(&config_path_full).unwrap();

    match hemtt_sqf::parser::run(&database, &processed) {
        Ok(mut sqf) => {
            if ignore_inspector {
                sqf.testing_clear_issues();
            }
            let codes = analyze(
                &sqf,
                Some(&config),
                &processed,
                Arc::new(Addon::test_addon()),
                database.clone(),
            );
            let stdout = codes
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

analyze!(s03_static_typename, true);
analyze!(s04_command_case, true);
analyze!(s05_if_assign, true);
analyze!(s06_find_in_str, true);
analyze!(s07_select_parse_number, true);
analyze!(s08_format_args, true);
analyze!(s09_banned_command, true);
analyze!(s11_if_not_else, true);
analyze!(s12_invalid_args, false);
analyze!(s13_undefined, false);
analyze!(s14_unused, false);
analyze!(s15_shadowed, false);
analyze!(s16_not_private, false);

analyze!(s18_in_vehicle_check, false);