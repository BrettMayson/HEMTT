#![allow(clippy::unwrap_used)]

//! Lints that need a whole project to run, so they can't be covered by the
//! fixture tests in their own crates.

mod common;

use common::{fixture, run, write};
use hemtt_test::directory::TemporaryDirectory;
use sealed_test::prelude::*;

/// `workspace_simple` with its prefixes fixed
#[must_use]
fn project() -> TemporaryDirectory {
    let directory = fixture("workspace_simple");
    write("addons/main/$PBOPREFIX$", "y\\test\\addons\\main");
    write("addons/secondary/$PBOPREFIX$", "y\\test\\addons\\secondary");
    write("addons/secondary/config.cpp", "class CfgPatches {};\n");
    write("addons/main/init.sqf", "systemChat 'hi';\n");
    directory
}

fn main_config(required_version: &str, body: &str) {
    write(
        "addons/main/config.cpp",
        format!(
            "class CfgPatches {{\n    class test_main {{\n        units[] = {{}};\n        weapons[] = {{}};\n        requiredVersion = {required_version};\n        requiredAddons[] = {{}};\n    }};\n}};\n{body}"
        ),
    );
}

// L-S01

#[sealed_test]
fn s01_command_newer_than_required_version() {
    let _directory = project();
    main_config("2.10", "");
    // `isEqualRef` was added in 2.12
    write(
        "addons/main/new.sqf",
        "systemChat str (player isEqualRef player);\n",
    );
    let result = run(&["check"]);
    result.fails();
    result
        .prints("error[L-S01]")
        .prints("command `isEqualRef` requires version 2.12")
        .prints("`CfgPatches` entry requires version 2.10");
}

#[sealed_test]
fn s01_command_within_required_version() {
    let _directory = project();
    main_config("2.12", "");
    write(
        "addons/main/new.sqf",
        "systemChat str (player isEqualRef player);\n",
    );
    run(&["check"]).succeeds().does_not_print("L-S01");
}

// L-C16

#[sealed_test]
fn c16_missing_file_in_config() {
    let _directory = project();
    main_config(
        "2.10",
        "class CfgVehicles {\n    class test_thing {\n        picture = \"\\y\\test\\addons\\main\\data\\missing.paa\";\n    };\n};\n",
    );
    run(&["check"]).prints("L-C16").prints("data\\missing.paa");
}

#[sealed_test]
fn c16_existing_file_in_config() {
    let _directory = project();
    write("addons/main/data/present.paa", "not really a paa");
    main_config(
        "2.10",
        "class CfgVehicles {\n    class test_thing {\n        picture = \"\\y\\test\\addons\\main\\data\\present.paa\";\n    };\n};\n",
    );
    run(&["check"]).does_not_print("L-C16");
}

#[sealed_test]
fn c16_forward_slashes_are_reported() {
    let _directory = project();
    write("addons/main/data/present.paa", "not really a paa");
    main_config(
        "2.10",
        "class CfgVehicles {\n    class test_thing {\n        picture = \"/y/test/addons/main/data/present.paa\";\n    };\n};\n",
    );
    run(&["check"])
        .prints("L-C16")
        .prints("Forward slashes are not valid in paths");
}

#[sealed_test]
fn c16_files_outside_the_project_are_ignored() {
    let _directory = project();
    main_config(
        "2.10",
        "class CfgVehicles {\n    class test_thing {\n        picture = \"\\a3\\ui_f\\data\\icon.paa\";\n    };\n};\n",
    );
    run(&["check"]).does_not_print("L-C16");
}
