#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{analyze::analyze, parser::database::Database};
use hemtt_workspace::{addons::Addon, reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/lints/";

macro_rules! lint {
    ($dir:ident, $ignore:expr) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                insta::assert_snapshot!(lint(stringify!($dir), $ignore));
            }
        }
    };
}

lint!(s02_event_handler_case, true);
lint!(s03_static_typename, true);
lint!(s04_command_case, true);
lint!(s05_if_assign, true);
lint!(s05_if_assign_emoji, true);
lint!(s06_find_in_str, true);
lint!(s07_select_parse_number, true);
lint!(s08_format_args, true);
lint!(s09_banned_command, true);
lint!(s11_if_not_else, true);
lint!(s12_invalid_args, false);
lint!(s13_undefined, false);
lint!(s14_unused, false);
lint!(s15_shadowed, false);
lint!(s16_not_private, false);
lint!(s17_var_all_caps, true);
lint!(s18_in_vehicle_check, true);
lint!(s19_extra_not, true);
lint!(s20_bool_static_comparison, true);
lint!(s21_invalid_comparisons, true);
lint!(s22_this_call, true);
lint!(s23_reassign_reserved_variable, true);
lint!(s24_marker_spam, true);

fn lint(file: &str, ignore_inspector: bool) -> String {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
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
            codes
                .iter()
                .map(|e| e.diagnostic().unwrap().to_string(&workspace_files))
                .collect::<Vec<_>>()
                .join("\n")
                .replace('\r', "")
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
    }
}
