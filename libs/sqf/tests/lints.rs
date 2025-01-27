#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{analyze::analyze, parser::database::Database};
use hemtt_workspace::{addons::Addon, reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/lints/";

macro_rules! lint {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                insta::assert_snapshot!(lint(stringify!($dir)));
            }
        }
    };
}

lint!(s02_event_handler_case);
lint!(s03_static_typename);
lint!(s04_command_case);
lint!(s05_if_assign);
lint!(s05_if_assign_emoji);
lint!(s06_find_in_str);
lint!(s07_select_parse_number);
lint!(s08_format_args);
lint!(s09_banned_command);
lint!(s11_if_not_else);
lint!(s17_var_all_caps);
lint!(s18_in_vehicle_check);
lint!(s19_extra_not);
lint!(s20_bool_static_comparison);
lint!(s21_invalid_comparisons);
lint!(s22_this_call);
lint!(s23_reassign_reserved_variable);
lint!(s24_marker_spam);
lint!(s27_localize_stringtable);
lint!(s28_banned_macros);

fn lint(file: &str) -> String {
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
    // let build_info = BuildInfo::new(config.prefix()).with_release(true);
    // let _ = build_info.stringtable_append(&format!("str_{}_validEntry", config.prefix()));

    match hemtt_sqf::parser::run(&database, &processed) {
        Ok(sqf) => {
            let (codes, _) = analyze(
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
