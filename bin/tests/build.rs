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

#[sealed_test]
fn build_sqf_too_many_constants() {
    let _directory = hemtt_test::directory::TemporaryDirectory::copy(&std::path::PathBuf::from(
        format!("{}/tests/workspace_simple/", env!("CARGO_MANIFEST_DIR")),
    ));
    // More unique constants than SQFC can index, the file should fall back to plain SQF
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
    hemtt::execute(&Cli::parse_from(vec!["hemtt", "build"])).unwrap();

    let pbo = fs_err::read(".hemttout/build/addons/test_main.pbo").expect("pbo was built");
    let contains = |needle: &[u8]| pbo.windows(needle.len()).any(|w| w == needle);
    assert!(contains(b"large.sqf\0"), "plain sqf is packed");
    assert!(!contains(b"large.sqfc"), "no sqfc for the large file");
    assert!(contains(b"init.sqfc"), "other files are still compiled");
}
