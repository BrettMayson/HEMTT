#![allow(clippy::unwrap_used)]

use hemtt_common::config::ProjectConfig;
use hemtt_config::ConfigReport;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{LayerType, reporting::WorkspaceFiles};

const ROOT: &str = "tests/lints/";

macro_rules! lint {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<config_error_ $dir>]() {
                insta::assert_snapshot!(lint(stringify!($dir)).0);
            }
        }
    };
}

lint!(c01_invalid_value);
lint!(c01m_invalid_value_macro);
lint!(c02_duplicate_property_shadow_property);
lint!(c03_duplicate_class);
lint!(c03_duplicate_external);
lint!(c04_missing_parent);
lint!(c05_parent_case);
lint!(c06_unexpected_array);
lint!(c07_expected_array_comment);
lint!(c07_expected_array);
lint!(c08_class_missing_final_brace);
lint!(c08_missing_semicolon);
// c09_magwell_missing_magazine is handled bellow
lint!(c10_class_missing_braces);
lint!(c11_file_type);
lint!(c12_math_could_be_unquoted);
lint!(c13_config_this_call);
lint!(c14_unused_external);
lint!(c14_macro_external);
lint!(c14_many_unused_external);
lint!(c15_cfgpatches_scope);
lint!(c17_extra_semicolon);

fn lint(file: &str) -> (String, ConfigReport) {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(
            Some(ProjectConfig::test_project()),
            false,
            &hemtt_common::config::PDriveOption::Disallow,
        )
        .unwrap();
    let source = workspace.join(format!("{file}.hpp")).unwrap();
    let processed = Processor::run(
        &source,
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .unwrap();
    let config_path_full = std::path::PathBuf::from(ROOT).join("project_tests.toml");
    let test_config = ProjectConfig::from_file(&config_path_full).unwrap();
    let parsed = hemtt_config::parse(Some(&test_config), &processed);
    let workspacefiles = WorkspaceFiles::new();
    match parsed {
        Ok(config) => (
            config
                .codes()
                .iter()
                // a code may deliberately have no diagnostic
                .filter_map(|e| e.diagnostic())
                .filter(|d| d.code != "L-C16")
                .map(|d| d.to_string(&workspacefiles))
                .collect::<Vec<_>>()
                .join("\n")
                .replace('\r', ""),
            config,
        ),
        // Errors may occur, but they should be handled, if one is not a handler should be created
        Err(e) => {
            for e in &e {
                eprintln!("{}", e.diagnostic().unwrap().to_string(&workspacefiles));
            }
            panic!("Error parsing config");
        }
    }
}

#[test]
/// Test `C09_gwell_missing_magazine` - maChecking results from the report (will not create errors directly)
fn test_c09_magwell_missing_magazine() {
    let (_, report) = lint(stringify!(c09_magwell_missing_magazine));
    insta::assert_compact_debug_snapshot!(report.magazine_well_info());
}

/// Regression for #1310: C14's summary count is per-run, not per-process.
#[test]
fn regression_1310_c14_count_is_per_run() {
    let first = lint(stringify!(c14_many_unused_external)).0;
    let second = lint(stringify!(c14_many_unused_external)).0;
    assert_eq!(first, second);
    assert!(
        first.contains("There are 7 unused external classes"),
        "expected a summary of all 7 unused externals, got:\n{first}"
    );
}

/// Regression for #1310: one config's count must not leak into the next.
#[test]
fn regression_1310_c14_count_not_shared_between_configs() {
    let _ = lint(stringify!(c14_many_unused_external));
    let few = lint(stringify!(c14_macro_external)).0;
    // under the summary threshold, so each is reported individually
    assert!(
        !few.contains("unused external classes"),
        "expected individual diagnostics, got a summary:\n{few}"
    );
    assert_eq!(few.matches("is never used").count(), 3);
}

#[test]
fn test_collect_cfgfunctions() {
    let (_, report) = lint(stringify!(collect_cfgfunctions));
    let mut functions_defined: Vec<&String> =
        report.functions_defined().iter().map(|(s, _)| s).collect();
    functions_defined.sort();
    insta::assert_compact_debug_snapshot!(functions_defined);
}
