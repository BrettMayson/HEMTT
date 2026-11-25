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
    // `init.sqf` can be compiled, but is excluded (e.g. it uses `#if __GAME_VER_MIN__`)
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
