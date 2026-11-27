#![allow(clippy::unwrap_used)]

//! `hemtt check` end to end: which problems fail the command, and how
//! `--error-on-all`, `--pedantic` and the project's lint config change that.

mod common;

use common::{fixture, run, write};
use hemtt::Error;
use hemtt_test::directory::TemporaryDirectory;
use sealed_test::prelude::*;

/// `workspace_simple` with its warnings fixed, so it checks with no diagnostics
#[must_use]
fn clean_workspace() -> TemporaryDirectory {
    let directory = fixture("workspace_simple");
    write("addons/main/$PBOPREFIX$", "y\\test\\addons\\main");
    write("addons/secondary/$PBOPREFIX$", "y\\test\\addons\\secondary");
    let config = common::read("addons/main/config.cpp")
        .replace("\\a\\b\\addons\\secondary", "\\y\\test\\addons\\secondary");
    write("addons/main/config.cpp", config);
    write(
        "addons/main/init.sqf",
        "private _test = 0;\nprivate _text = \"\";\n{\n    _text = str (_test + _x);\n} forEach [1,2,3,4];\nsystemChat _text;\n",
    );
    directory
}

#[sealed_test]
fn clean_project_passes_even_with_error_on_all() {
    let _directory = clean_workspace();
    run(&["check"])
        .succeeds()
        .does_not_print("warning[")
        .does_not_print("error[");
    run(&["check", "--error-on-all"]).succeeds();
}

#[sealed_test]
fn sqf_syntax_error_fails() {
    let _directory = clean_workspace();
    write("addons/main/broken.sqf", "private _a = ;\n");
    let result = run(&["check"]);
    assert!(matches!(result.fails(), Error::ReportFailed));
    result
        .prints("error[SPE2]")
        .prints("addons/main/broken.sqf:1:12");
}

#[sealed_test]
fn config_error_fails() {
    let _directory = clean_workspace();
    write(
        "addons/main/config.cpp",
        "class CfgPatches {\n    class test_main {\n        units[] = {};\n    };\n};\nclass Broken {\n    value = 1\n};\n",
    );
    let result = run(&["check"]);
    result.fails();
    result.prints("addons/main/config.cpp");
}

#[sealed_test]
fn preprocessor_error_fails() {
    let _directory = clean_workspace();
    write(
        "addons/main/config.cpp",
        "#include \"does_not_exist.hpp\"\nclass CfgPatches {};\n",
    );
    let result = run(&["check"]);
    result.fails();
    result.prints("does_not_exist.hpp");
}

#[sealed_test]
fn warnings_only_fail_with_error_on_all() {
    // `workspace_simple` reports pboprefix warnings
    let _directory = fixture("workspace_simple");
    run(&["check"])
        .succeeds()
        .prints("warning[INVALID-PBOPREFIX]");
    let result = run(&["check", "--error-on-all"]);
    assert!(matches!(result.fails(), Error::ReportFailed));
}

#[sealed_test]
fn help_only_fails_with_error_on_all() {
    let _directory = clean_workspace();
    write("addons/main/undefined.sqf", "systemChat str _undefined;\n");
    run(&["check"]).succeeds().prints("help[L-S13]");
    run(&["check", "-e"]).fails();
}

#[sealed_test]
fn pedantic_enables_more_lints() {
    let _directory = clean_workspace();
    write("addons/main/unused.sqf", "private _unused = 1;\n");
    run(&["check"]).succeeds().does_not_print("L-S14");
    run(&["check", "--pedantic"])
        .succeeds()
        .prints("help[L-S14]")
        .prints("Unused variable `_unused`");
}

#[sealed_test]
fn project_config_disables_a_lint() {
    let _directory = clean_workspace();
    write("addons/main/undefined.sqf", "systemChat str _undefined;\n");
    let project = common::read(".hemtt/project.toml");
    write(
        ".hemtt/project.toml",
        format!("{project}\n[lints.sqf]\nundefined = false\n"),
    );
    run(&["check", "-e"]).succeeds().does_not_print("L-S13");
}

#[sealed_test]
fn project_config_raises_a_lint_to_error() {
    let _directory = clean_workspace();
    write("addons/main/undefined.sqf", "systemChat str _undefined;\n");
    let project = common::read(".hemtt/project.toml");
    write(
        ".hemtt/project.toml",
        format!("{project}\n[lints.sqf]\nundefined = \"Error\"\n"),
    );
    let result = run(&["check"]);
    result.fails();
    result.prints("error[L-S13]");
}

#[sealed_test]
fn pedantic_does_not_override_disabled_lints() {
    let _directory = clean_workspace();
    write("addons/main/unused.sqf", "private _unused = 1;\n");
    let project = common::read(".hemtt/project.toml");
    write(
        ".hemtt/project.toml",
        format!("{project}\n[lints.sqf]\nunused = false\n"),
    );
    run(&["check", "--pedantic"])
        .succeeds()
        .does_not_print("L-S14");
}

#[sealed_test]
fn unknown_lint_in_config_is_reported() {
    let _directory = clean_workspace();
    let project = common::read(".hemtt/project.toml");
    write(
        ".hemtt/project.toml",
        format!("{project}\n[lints.sqf]\nnot_a_real_lint = false\n"),
    );
    run(&["check"]).prints("not_a_real_lint");
}

#[sealed_test]
fn check_does_not_write_build_output() {
    let _directory = clean_workspace();
    run(&["check"]).succeeds();
    assert!(!std::path::Path::new(".hemttout/build").exists());
    assert!(!std::path::Path::new(".hemttout/dev").exists());
    assert!(!std::path::Path::new(".hemttout/release").exists());
}

#[sealed_test]
fn missing_project_config_fails() {
    let _directory = TemporaryDirectory::new();
    write("addons/main/config.cpp", "class CfgPatches {};\n");
    run(&["check"]).fails();
}

// `-L`, explicit lints

const IF_NOT_ELSE: &str = "if (!alive player) then { systemChat 'a' } else { systemChat 'b' };\n";

#[sealed_test]
fn explicit_lint_enables_a_disabled_lint() {
    // S11 is disabled by default, not even `--pedantic` enables it
    let _directory = clean_workspace();
    write("addons/main/not.sqf", IF_NOT_ELSE);
    run(&["check", "--pedantic"])
        .succeeds()
        .does_not_print("L-S11");
    for name in ["S11", "s11", "L-S11", "l-s11", "if_not_else", "IF_NOT_ELSE"] {
        run(&["check", "-L", name]).succeeds().prints("help[L-S11]");
    }
}

#[sealed_test]
fn explicit_lint_only_enables_the_named_lint() {
    let _directory = clean_workspace();
    write("addons/main/not.sqf", IF_NOT_ELSE);
    write("addons/main/unused.sqf", "private _unused = 1;\n");
    run(&["check", "-L", "S11"])
        .succeeds()
        .prints("L-S11")
        .does_not_print("L-S14");
    run(&["check", "-L", "S11", "-L", "unused"])
        .succeeds()
        .prints("L-S11")
        .prints("L-S14");
}

#[sealed_test]
fn explicit_lint_overrides_project_config() {
    let _directory = clean_workspace();
    write("addons/main/unused.sqf", "private _unused = 1;\n");
    let project = common::read(".hemtt/project.toml");
    write(
        ".hemtt/project.toml",
        format!("{project}\n[lints.sqf]\nunused = false\n"),
    );
    run(&["check", "-L", "S14"]).succeeds().prints("L-S14");
}

#[sealed_test]
fn explicit_lint_keeps_configured_severity() {
    let _directory = clean_workspace();
    write("addons/main/not.sqf", IF_NOT_ELSE);
    let project = common::read(".hemtt/project.toml");
    write(
        ".hemtt/project.toml",
        format!("{project}\n[lints.sqf.if_not_else]\nseverity = \"Error\"\nenabled = false\n"),
    );
    let result = run(&["check", "-L", "S11"]);
    result.fails();
    result.prints("error[L-S11]");
}

#[sealed_test]
fn explicit_lint_unknown_fails_with_suggestion() {
    let _directory = clean_workspace();
    let result = run(&["check", "-L", "if_not_els"]);
    assert!(matches!(result.fails(), Error::ReportFailed));
    result
        .prints("No lint found for `if_not_els`")
        .prints("if_not_else");
}

#[sealed_test]
fn explicit_config_lint() {
    // C14 lists every unused external class instead of a summary when named
    let _directory = clean_workspace();
    let externals = (0..8)
        .map(|i| format!("class External{i};\n"))
        .collect::<Vec<_>>()
        .concat();
    let config = common::read("addons/main/config.cpp");
    write("addons/main/config.cpp", format!("{externals}{config}"));
    run(&["check"])
        .succeeds()
        .prints("There are 8 unused external classes")
        .prints("hemtt check -Lc14");
    let explicit = run(&["check", "-Lc14"]);
    explicit.succeeds().does_not_print("There are 8");
    for i in 0..8 {
        explicit.prints(&format!("external class External{i} is never used"));
    }
}
