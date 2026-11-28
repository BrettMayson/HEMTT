#![allow(clippy::unwrap_used)]

//! `hemtt utils` end to end, on PBOs and files produced by a real build.
//!
//! Every util must fail, not just log an error, when it can't do its job,
//! so scripts and CI can rely on the exit status.

mod common;

use std::{io::Cursor, path::Path};

use common::{fixture, read, run, write};
use hemtt::Error;
use hemtt_pbo::WritablePbo;
use hemtt_test::directory::TemporaryDirectory;
use sealed_test::prelude::*;

const MAIN_PBO: &str = ".hemttout/build/addons/test_main.pbo";

#[must_use]
fn built() -> TemporaryDirectory {
    let directory = fixture("workspace_simple");
    run(&["build"]).succeeds();
    assert!(Path::new(MAIN_PBO).exists());
    directory
}

fn write_pbo(path: &str, files: &[(&str, &[u8])]) {
    let mut pbo = WritablePbo::new();
    for (name, data) in files {
        pbo.add_file(*name, Cursor::new(data.to_vec())).unwrap();
    }
    pbo.add_property("prefix", "z\\test\\addons\\crafted");
    let mut out = Vec::new();
    pbo.write(&mut out, true).unwrap();
    fs_err::write(path, out).unwrap();
}

// pbo

#[sealed_test]
fn pbo_inspect() {
    let _directory = built();
    run(&["utils", "pbo", "inspect", MAIN_PBO])
        .succeeds()
        .prints("config.bin")
        .prints("init.sqf");
}

#[sealed_test]
fn pbo_inspect_not_a_pbo_fails() {
    let _directory = built();
    write("not.pbo", "this is not a pbo");
    run(&["utils", "pbo", "inspect", "not.pbo"]).fails();
}

#[sealed_test]
fn pbo_unpack() {
    let _directory = built();
    run(&["utils", "pbo", "unpack", MAIN_PBO, "unpacked"]).succeeds();
    assert_eq!(
        read("unpacked/init.sqf"),
        read("addons/main/init.sqf"),
        "sqf is packed unchanged"
    );
    assert!(Path::new("unpacked/config.bin").exists());
    assert!(read("unpacked/$PBOPREFIX$").contains("addons\\main"));
}

#[sealed_test]
fn pbo_unpack_derap() {
    let _directory = built();
    run(&["utils", "pbo", "unpack", "--derap", MAIN_PBO, "unpacked"]).succeeds();
    let config = read("unpacked/config.cpp");
    assert!(config.contains("class MyMod"), "{config}");
    assert!(config.contains("yes it is my mod"), "{config}");
}

#[sealed_test]
fn pbo_unpack_existing_output_fails() {
    let _directory = built();
    fs_err::create_dir("unpacked").unwrap();
    let result = run(&["utils", "pbo", "unpack", MAIN_PBO, "unpacked"]);
    assert!(matches!(result.fails(), Error::OutputExists(_)));
}

#[sealed_test]
fn pbo_unpack_stays_inside_the_output_folder() {
    let _directory = TemporaryDirectory::new();
    fs_err::create_dir("work").unwrap();
    write_pbo(
        "work/crafted.pbo",
        &[("..\\..\\escaped.txt", b"outside"), ("safe.txt", b"inside")],
    );
    let result = run(&["utils", "pbo", "unpack", "work/crafted.pbo", "work/out"]);
    assert!(matches!(result.fails(), Error::UnsafePboPath(_)));
    assert!(
        !Path::new("escaped.txt").exists() && !Path::new("work/escaped.txt").exists(),
        "a PBO entry wrote outside the output folder"
    );
}

#[sealed_test]
fn pbo_extract() {
    let _directory = built();
    run(&[
        "utils",
        "pbo",
        "extract",
        MAIN_PBO,
        "init.sqf",
        "init_out.sqf",
    ])
    .succeeds();
    assert_eq!(read("init_out.sqf"), read("addons/main/init.sqf"));
}

#[sealed_test]
fn pbo_extract_missing_file_fails() {
    let _directory = built();
    let result = run(&["utils", "pbo", "extract", MAIN_PBO, "nope.sqf", "out.sqf"]);
    assert!(matches!(result.fails(), Error::NotFound(_)));
    assert!(!Path::new("out.sqf").exists());
}

#[sealed_test]
fn pbo_extract_existing_output_fails() {
    let _directory = built();
    write("out.sqf", "keep me");
    let result = run(&["utils", "pbo", "extract", MAIN_PBO, "init.sqf", "out.sqf"]);
    assert!(matches!(result.fails(), Error::OutputExists(_)));
    assert_eq!(read("out.sqf"), "keep me", "the existing file is untouched");
}

// config

#[sealed_test]
fn config_derapify_roundtrip() {
    let _directory = built();
    run(&[
        "utils",
        "pbo",
        "extract",
        MAIN_PBO,
        "config.bin",
        "config.bin",
    ])
    .succeeds();
    run(&["utils", "config", "derapify", "config.bin", "config.cpp"]).succeeds();
    let config = read("config.cpp");
    assert!(config.contains("class MyMod"), "{config}");
    assert!(config.contains("yes it is my mod"), "{config}");
}

#[sealed_test]
fn config_derapify_json() {
    let _directory = built();
    run(&[
        "utils",
        "pbo",
        "extract",
        MAIN_PBO,
        "config.bin",
        "config.bin",
    ])
    .succeeds();
    run(&[
        "utils",
        "config",
        "derapify",
        "-f",
        "json",
        "config.bin",
        "config.json",
    ])
    .succeeds();
    let json: serde_json::Value = serde_json::from_str(&read("config.json")).unwrap();
    assert!(json.to_string().contains("yes it is my mod"), "{json}");
}

#[sealed_test]
fn config_derapify_not_rapified_fails() {
    let _directory = TemporaryDirectory::new();
    write("config.bin", "class NotRapified {};");
    run(&["utils", "config", "derapify", "config.bin", "config.cpp"]).fails();
}

// text fixes

#[sealed_test]
fn fnl_adds_missing_final_newlines() {
    let _directory = TemporaryDirectory::new();
    write("addons/main/no_newline.sqf", "systemChat 'hi';");
    write("addons/main/has_newline.sqf", "systemChat 'hi';\n");
    write("addons/main/empty.sqf", "");
    write("addons/main/binary.paa", "no newline");
    run(&["utils", "fnl"]).succeeds();
    assert_eq!(read("addons/main/no_newline.sqf"), "systemChat 'hi';\n");
    assert_eq!(read("addons/main/has_newline.sqf"), "systemChat 'hi';\n");
    assert_eq!(
        read("addons/main/empty.sqf"),
        "",
        "empty files are left alone"
    );
    assert_eq!(
        read("addons/main/binary.paa"),
        "no newline",
        "only text files"
    );

    // Running again changes nothing
    run(&["utils", "fnl"]).succeeds();
    assert_eq!(read("addons/main/no_newline.sqf"), "systemChat 'hi';\n");
}

#[sealed_test]
fn bom_removes_byte_order_marks() {
    let _directory = TemporaryDirectory::new();
    write("addons/main/bom.sqf", b"\xEF\xBB\xBFsystemChat 'hi';\n");
    write("addons/main/plain.sqf", "systemChat 'hi';\n");
    run(&["utils", "bom"]).succeeds();
    assert_eq!(read("addons/main/bom.sqf"), "systemChat 'hi';\n");
    assert_eq!(read("addons/main/plain.sqf"), "systemChat 'hi';\n");
}

#[sealed_test]
fn bom_fix_satisfies_check() {
    // The check's BOM warning tells users to run `hemtt utils bom`, so that must fix it
    let _directory = fixture("workspace_simple");
    let init = fs_err::read("addons/main/init.sqf").unwrap();
    write(
        "addons/main/init.sqf",
        [b"\xEF\xBB\xBF".as_slice(), &init].concat(),
    );
    run(&["check"]).prints("utils bom");
    run(&["utils", "bom"]).succeeds();
    run(&["check"]).does_not_print("utils bom");
}

#[sealed_test]
fn sqf_case_fixes_command_case() {
    let _directory = TemporaryDirectory::new();
    write(
        "script.sqf",
        "private _x = SYSTEMCHAT \"systemchat stays\"; // systemchat stays\nHINT str _x;\n",
    );
    run(&["utils", "sqf", "case", "script.sqf"]).succeeds();
    assert_eq!(
        read("script.sqf"),
        "private _x = systemChat \"systemchat stays\"; // systemchat stays\nhint str _x;\n"
    );
}

#[sealed_test]
fn sqf_case_missing_file_fails() {
    let _directory = TemporaryDirectory::new();
    run(&["utils", "sqf", "case", "nope.sqf"]).fails();
}

// paa

#[sealed_test]
fn paa_compress_missing_path_fails() {
    let _directory = TemporaryDirectory::new();
    let result = run(&["utils", "paa", "compress", "does_not_exist"]);
    assert!(matches!(result.fails(), Error::NotFound(_)));
}

// inspect

#[sealed_test]
fn inspect_detects_file_types() {
    let _directory = built();
    run(&["utils", "inspect", MAIN_PBO])
        .succeeds()
        .prints("init.sqf");
    run(&[
        "utils",
        "pbo",
        "extract",
        MAIN_PBO,
        "config.bin",
        "config.bin",
    ])
    .succeeds();
    run(&["utils", "inspect", "config.bin"]).succeeds();
}

#[sealed_test]
fn inspect_missing_file_fails() {
    let _directory = TemporaryDirectory::new();
    run(&["utils", "inspect", "nope.pbo"]).fails();
}
