#![allow(clippy::unwrap_used)]

use std::{io::Read, sync::Arc};

use hemtt_common::config::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{analyze::analyze, parser::database::Database};
use hemtt_workspace::{addons::Addon, reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/lints/";

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
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("source.sqf").unwrap();
    let processed = Processor::run(&source).unwrap();
    let database = Arc::new(Database::a3(false));
    let workspace_files = WorkspaceFiles::new();

    let config_path_full = std::path::PathBuf::from(ROOT).join("project_tests.toml");
    let config = ProjectConfig::from_file(&config_path_full).unwrap();

    match hemtt_sqf::parser::run(&database, &processed) {
        Ok(sqf) => {
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

analyze!(s02_event_handler_case);
analyze!(s03_static_typename);
analyze!(s04_command_case);
analyze!(s05_if_assign);
analyze!(s06_find_in_str);
analyze!(s07_select_parse_number);
analyze!(s08_format_args);
analyze!(s09_banned_command);
analyze!(s11_if_not_else);
analyze!(s17_var_all_caps);
analyze!(s18_in_vehicle_check);
analyze!(s20_bool_static_comparison);
analyze!(s21_invalid_comparisons);
analyze!(s22_this_call);
analyze!(s23_reassign_reserved_variable);
