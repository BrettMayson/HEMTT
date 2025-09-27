#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{
    analyze::{SqfReport, analyze},
    parser::database::Database,
};
use hemtt_workspace::{LayerType, addons::Addon, position::Position, reporting::WorkspaceFiles};

const ROOT: &str = "tests/lints/";

macro_rules! lint {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                insta::assert_snapshot!(lint(stringify!($dir)).0);
            }
        }
    };
}

lint!(s02_event_handler_case);
lint!(s03_static_typename);
lint!(s04_command_case);
lint!(s05_if_assign_emoji);
lint!(s05_if_assign);
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
lint!(s26_short_circuit_bool_var);
lint!(s27_select_count);
lint!(s28_banned_macros);
lint!(s30_configof);

#[test]
fn test_s29_function_undefined() {
    let (_, report) = lint(stringify!(s29_undefined_functions));
    let mut functions_defined: Vec<&String> =
        report.functions_defined().iter().map(|(s, _)| s).collect();
    functions_defined.sort();
    let mut functions_used: Vec<(&String, &Position)> = report
        .functions_used()
        .iter()
        .map(|fu| (&fu.0, &fu.1))
        .collect();
    functions_used.sort_by(|a, b| a.0.cmp(b.0));
    insta::assert_compact_debug_snapshot!((functions_defined, functions_used));
}

fn lint(file: &str) -> (String, SqfReport) {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(
        &source,
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .unwrap();
    let database = Arc::new(Database::a3(false));
    let workspace_files = WorkspaceFiles::new();

    let config_path_full = std::path::PathBuf::from(ROOT).join("project_tests.toml");
    let config = ProjectConfig::from_file(&config_path_full).unwrap();

    match hemtt_sqf::parser::run(&database, &processed) {
        Ok(sqf) => {
            let (codes, report) = analyze(
                &sqf,
                Some(&config),
                &processed,
                Arc::new(Addon::test_addon()),
                database.clone(),
            );
            (
                codes
                    .iter()
                    .map(|e| e.diagnostic().unwrap().to_string(&workspace_files))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace('\r', ""),
                report.expect("exist"),
            )
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
