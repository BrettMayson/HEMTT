#![allow(clippy::unwrap_used)]

use hemtt_common::config::PreprocessorOptions;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{LayerType, reporting::Processed};

fn run(file: &str, options: &PreprocessorOptions) -> Processed {
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&std::path::PathBuf::from("tests/debug"), LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    Processor::run(&workspace.join(file).unwrap(), options).unwrap()
}

#[test]
fn ifdef_debug() {
    let normal = run("ifdef.sqf", &PreprocessorOptions::default());
    assert!(normal.uses_debug_macro());
    assert!(normal.as_str().contains("\"not debug\""));
    assert!(!normal.as_str().contains("\"debug\""));

    let debug = run(
        "ifdef.sqf",
        &PreprocessorOptions::default().with_debug(true),
    );
    assert!(debug.uses_debug_macro());
    assert!(debug.as_str().contains("\"debug\""));
    assert!(!debug.as_str().contains("\"not debug\""));
}

#[test]
fn if_debug() {
    // `#if` on a runtime macro needs `runtime_macros`, which defines it as 0
    let normal = run(
        "if.sqf",
        &PreprocessorOptions::default().with_runtime_macros(true),
    );
    assert!(normal.uses_debug_macro());
    assert!(!normal.as_str().contains("\"debug\""));

    let debug = run(
        "if.sqf",
        &PreprocessorOptions::default()
            .with_runtime_macros(true)
            .with_debug(true),
    );
    assert!(debug.as_str().contains("\"debug\""));
    assert!(debug.as_str().contains("\"shared\""));
}

#[test]
fn no_debug_macro() {
    assert!(!run("none.sqf", &PreprocessorOptions::default()).uses_debug_macro());
    assert!(!run("none.sqf", &PreprocessorOptions::default().with_debug(true)).uses_debug_macro());
}
