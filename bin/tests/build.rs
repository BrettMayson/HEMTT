#![allow(clippy::unwrap_used)]

use clap::Parser;
use sealed_test::prelude::*;

use hemtt::Cli;

#[sealed_test]
fn build_simple() {
    let _directory = hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(
        format!("{}/tests/workspace_simple/", env!("CARGO_MANIFEST_DIR")),
    ));
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "dev"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "build"])).unwrap();
}

#[sealed_test]
fn build_post_release() {
    let _directory =
        hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(format!(
            "{}/tests/workspace_post_release/",
            env!("CARGO_MANIFEST_DIR")
        )));
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "script", "test"])).unwrap();
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "release"])).unwrap();
}

/// Build `workspace_simple`, optionally with a file that cannot be compiled to SQFC
/// and an `[sqfc]` section in the main addon's `addon.toml`
///
/// Returns the main PBO if it was built, and the command output
fn build_with_sqfc(large_file: bool, sqfc_section: &str) -> (Option<Vec<u8>>, String) {
    if large_file {
        // More unique constants than SQFC can index
        let items = (0..70_000)
            .map(|i| format!("{i}.5"))
            .collect::<Vec<_>>()
            .chunks(1000)
            .map(|chunk| format!("[{}]", chunk.join(",")))
            .collect::<Vec<_>>()
            .join(",\n");
        fs_err::write(
            "addons/main/large.sqf",
            format!("large_data = [\n{items}\n];\n"),
        )
        .unwrap();
    }
    let addon_toml = fs_err::read_to_string("addons/main/addon.toml").unwrap();
    fs_err::write(
        "addons/main/addon.toml",
        format!("{addon_toml}\n[sqfc]\n{sqfc_section}\n"),
    )
    .unwrap();
    let capture = hemtt_common::capture::OutputCapture::new();
    let _ = hemtt::execute(&Cli::parse_from(vec!["hemtt", "build"]));
    let output = capture.finish();
    (
        fs_err::read(".hemttout/build/addons/test_main.pbo").ok(),
        output,
    )
}

fn pbo_contains(pbo: &[u8], needle: &[u8]) -> bool {
    pbo.windows(needle.len()).any(|w| w == needle)
}

fn copy_simple() -> hemtt_test::directory::TemporaryDirectory {
    hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(format!(
        "{}/tests/workspace_simple/",
        env!("CARGO_MANIFEST_DIR")
    )))
}

#[sealed_test]
fn build_sqfc_failure_is_an_error() {
    let _directory = copy_simple();
    let (pbo, output) = build_with_sqfc(true, "");
    assert!(pbo.is_none(), "the build fails");
    assert!(
        output.contains("large.sqf` could not be compiled to SQFC"),
        "{output}"
    );
}

#[sealed_test]
fn build_sqfc_exclude() {
    let _directory = copy_simple();
    // `init.sqf` can be compiled, but is excluded
    let (pbo, output) = build_with_sqfc(true, r#"exclude = ["large.sqf", "init.sqf"]"#);
    let pbo = pbo.expect("pbo was built");
    assert!(pbo_contains(&pbo, b"large.sqf\0"), "plain sqf is packed");
    assert!(
        !pbo_contains(&pbo, b"large.sqfc"),
        "no sqfc for excluded file"
    );
    assert!(pbo_contains(&pbo, b"init.sqf\0"), "plain sqf is packed");
    assert!(
        !pbo_contains(&pbo, b"init.sqfc"),
        "no sqfc for excluded file"
    );
    assert!(!output.contains("SQFC"), "{output}");
}

#[sealed_test]
fn build_sqfc_exclude_temporary() {
    let _directory = copy_simple();
    let (pbo, output) = build_with_sqfc(true, r#"exclude_temporary = ["large.sqf"]"#);
    let pbo = pbo.expect("pbo was built");
    assert!(pbo_contains(&pbo, b"large.sqf\0"), "plain sqf is packed");
    assert!(
        !pbo_contains(&pbo, b"large.sqfc"),
        "no sqfc for excluded file"
    );
    assert!(
        pbo_contains(&pbo, b"init.sqfc"),
        "other files are still compiled"
    );
    assert!(
        !output.contains("SQFC"),
        "still needed, so no warning: {output}"
    );
}

#[sealed_test]
fn build_sqfc_exclude_temporary_not_needed() {
    let _directory = copy_simple();
    let (pbo, output) = build_with_sqfc(false, r#"exclude_temporary = ["init.sqf"]"#);
    let pbo = pbo.expect("pbo was built");
    assert!(
        !pbo_contains(&pbo, b"init.sqfc"),
        "still excluded until removed from the list"
    );
    assert!(
        output.contains("init.sqf` is in `sqfc.exclude_temporary` but can be compiled to SQFC"),
        "{output}"
    );
}

#[sealed_test]
fn build_sqfc_line_limit() {
    let _directory = copy_simple();
    // SQFC stores line numbers as u16, a file this long can't be compiled
    fs_err::write(
        "addons/main/long.sqf",
        format!("early = 1;\n{}late = 2;\n", "\n".repeat(70_000)),
    )
    .unwrap();
    let (pbo, output) = build_with_sqfc(false, "");
    assert!(pbo.is_none(), "the build fails");
    assert!(
        output.contains("long.sqf` could not be compiled to SQFC"),
        "{output}"
    );
    assert!(output.contains("the limit is 65535"), "{output}");
}

#[sealed_test]
fn build_sqfc_line_limit_exclude_temporary() {
    let _directory = copy_simple();
    fs_err::write(
        "addons/main/long.sqf",
        format!("early = 1;\n{}late = 2;\n", "\n".repeat(70_000)),
    )
    .unwrap();
    let (pbo, output) = build_with_sqfc(false, r#"exclude_temporary = ["long.sqf"]"#);
    let pbo = pbo.expect("pbo was built");
    assert!(pbo_contains(&pbo, b"long.sqf\0"), "plain sqf is packed");
    assert!(
        !pbo_contains(&pbo, b"long.sqfc"),
        "no sqfc for excluded file"
    );
    assert!(!output.contains("SQFC"), "{output}");
}

const VERSION_CHECK: &str =
    "#if __GAME_VER_MIN__ == 22\nversion = 22;\n#else\nversion = 0;\n#endif\n";

#[sealed_test]
fn build_unsupported_builtin_in_if_is_an_error() {
    let _directory = copy_simple();
    fs_err::write("addons/main/version.sqf", VERSION_CHECK).unwrap();
    let capture = hemtt_common::capture::OutputCapture::new();
    let _ = hemtt::execute(&Cli::parse_from(vec!["hemtt", "build"]));
    let output = capture.finish();
    assert!(
        !std::path::Path::new(".hemttout/build/addons/test_main.pbo").exists(),
        "the build fails"
    );
    assert!(
        output.contains("built-in macro `__GAME_VER_MIN__` is not supported by HEMTT"),
        "{output}"
    );
}

#[sealed_test]
fn build_unsupported_builtin_pragma_sqf() {
    let _directory = copy_simple();
    fs_err::write(
        "addons/main/version.sqf",
        format!("#pragma hemtt flag pe26_ignore_unsupported_builtin\n{VERSION_CHECK}"),
    )
    .unwrap();
    let (pbo, output) = build_with_sqfc(false, "");
    let pbo = pbo.expect("pbo was built");
    assert!(pbo_contains(&pbo, b"version.sqf\0"), "plain sqf is packed");
    assert!(
        !pbo_contains(&pbo, b"version.sqfc"),
        "the game must evaluate the condition, so no sqfc"
    );
    assert!(
        pbo_contains(&pbo, b"init.sqfc"),
        "other files are still compiled"
    );
    assert!(!output.contains("PE26"), "{output}");
}

#[sealed_test]
fn build_unsupported_builtin_pragma_config() {
    let _directory = copy_simple();
    let config = fs_err::read_to_string("addons/main/config.cpp").unwrap();
    fs_err::write(
        "addons/main/config.cpp",
        format!(
            "#pragma hemtt flag pe26_ignore_unsupported_builtin\n#if __GAME_VER_MIN__ >= 22\n#define NEW_FEATURE 1\n#endif\n{config}"
        ),
    )
    .unwrap();
    let (pbo, output) = build_with_sqfc(false, "");
    let pbo = pbo.expect("pbo was built");
    assert!(
        pbo_contains(&pbo, b"config.cpp\0"),
        "config is not rapified"
    );
    assert!(
        !pbo_contains(&pbo, b"config.bin\0"),
        "config is not rapified"
    );
    assert!(!output.contains("PE26"), "{output}");
}

/// Read a file out of the built main PBO
fn pbo_file(name: &str) -> Vec<u8> {
    use std::io::Read;
    let mut pbo = hemtt_pbo::ReadablePbo::from(
        fs_err::File::open(".hemttout/build/addons/test_main.pbo").expect("pbo was built"),
    )
    .unwrap();
    let mut file = pbo.file(name).unwrap().expect("file in pbo");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

#[sealed_test]
fn build_sqfc_debug_block() {
    let _directory = copy_simple();
    fs_err::write(
        "addons/main/dbg.sqf",
        "#ifdef __A3_DEBUG__\nhint \"debug\";\n#else\nhint \"not debug\";\n#endif\n",
    )
    .unwrap();
    let (pbo, output) = build_with_sqfc(false, "");
    assert!(pbo.is_some(), "{output}");

    // ends with the code block (3), then the debug code block (4)
    let dbg = pbo_file("dbg.sqfc");
    assert_eq!(dbg[dbg.len() - 18], 3, "code block");
    assert_eq!(dbg[dbg.len() - 9], 4, "debug code block");

    // files that don't use `__A3_DEBUG__` have no debug block
    let init = pbo_file("init.sqfc");
    assert_eq!(init[init.len() - 9], 3, "ends with the code block");
}

#[sealed_test]
fn build_sqfc_debug_block_error() {
    let _directory = copy_simple();
    // only invalid when the game is started with `-debug`
    fs_err::write(
        "addons/main/dbg.sqf",
        "#ifdef __A3_DEBUG__\nhint \"debug\" +;\n#endif\nhint \"shared\";\n",
    )
    .unwrap();
    let (pbo, output) = build_with_sqfc(false, "");
    assert!(pbo.is_none(), "the build fails: {output}");
}
