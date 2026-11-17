use std::path::Path;

use clap::Parser;
use hemtt::commands::publish::{self, Command};
use sealed_test::prelude::*;

fn project() -> hemtt_test::directory::TemporaryDirectory {
    let directory = hemtt_test::directory::TemporaryDirectory::new();
    fs_err::create_dir_all(".hemtt").expect("configuration directory");
    fs_err::write(".hemtt/project.toml", "name = 'Publisher test'\nprefix = 'publish_test'\n[version]\nmajor = 1\nminor = 2\npatch = 3\ngit_hash = 0\n").expect("project config");
    fs_err::create_dir_all("addons/main").expect("addon directory");
    fs_err::write("addons/main/$PBOPREFIX$", "x\\publish_test\\addons\\main")
        .expect("addon prefix");
    fs_err::write(
        "addons/main/config.cpp",
        r#"class CfgPatches {
    class publish_test_main {
        requiredVersion = 2.20;
        requiredAddons[] = {"A3_Data_F"};
        units[] = {};
        weapons[] = {};
    };
};
"#,
    )
    .expect("addon config");
    directory
}

fn command(dry_run: bool) -> Command {
    let mut args = vec!["publish", "--no-bin", "--no-sign", "--no-archive"];
    if dry_run {
        args.push("--dry-run");
    }
    Command::parse_from(args)
}

#[sealed_test]
fn published_id_accepts_inline_and_multiline_declarations() {
    let _directory = hemtt_test::directory::TemporaryDirectory::new();
    for meta in [
        "protocol = 1;\npublishedid = 1234567890;\n",
        "protocol = 1; publishedid = 1234567890;",
        "/* Workshop */ publishedid = 1234567890;",
        "\t publishedid\t=\t1234567890\t;\r\n",
    ] {
        fs_err::write("meta.cpp", meta).expect("published ID");
        assert_eq!(
            publish::get_id().expect("valid published ID").0,
            1_234_567_890,
            "metadata: {meta}"
        );
    }
}

#[sealed_test]
fn published_id_rejects_invalid_values_and_prefixed_identifiers() {
    let _directory = hemtt_test::directory::TemporaryDirectory::new();
    for meta in [
        "publishedid = 0;",
        "publishedid = 18446744073709551616;",
        "publishedid = invalid;",
        "otherpublishedid = 1234567890;",
        "_publishedid = 1234567890;",
    ] {
        fs_err::write("meta.cpp", meta).expect("invalid published ID");
        assert!(publish::get_id().is_err(), "metadata: {meta}");
    }
}

#[sealed_test]
fn dry_run_builds_release_with_all_metadata_without_creating_a_workshop_item() {
    let _project = project();
    let image = image::DynamicImage::new_rgb8(2, 3);
    image.save("preview.png").expect("primary image");
    image.save("gallery.jpg").expect("gallery JPEG");
    image.save("gallery.gif").expect("gallery GIF");
    fs_err::write("DESCRIPTION.md", "# Example\n\n**Workshop** description").expect("description");
    fs_err::write(
        "CHANGELOG.md",
        "# Changelog\n\n## [1.2.3] - 2026-01-01\n\n- Test upload\n",
    )
    .expect("changelog");
    fs_err::write(
        ".hemtt/publish.toml",
        r#"
title = "Workshop override"
language = "english"
visibility = "private"
tags = ["Mod", "Editor Extension"]
description = "DESCRIPTION.md"
changelog = "CHANGELOG.md"
preview = "preview.png"
required_items = ["450814997", "463939057"]
additional_previews = [
    { type = "image", path = "preview.png" },
    { type = "image", path = "gallery.jpg" },
    { type = "image", path = "gallery.gif" },
    { type = "youtube", video_id = "dQw4w9WgXcQ" },
]
"#,
    )
    .expect("publish config");
    let report = publish::execute(&command(true)).expect("offline dry run");
    assert!(!report.failed(), "release build should pass");
    assert!(Path::new(".hemttout/release/addons/publish_test_main.pbo").is_file());
    assert!(
        !Path::new("meta.cpp").exists(),
        "dry run must not create a Workshop item"
    );
    // An existing item follows the same offline validation and build path.
    for meta in [
        "protocol = 1;\npublishedid = 1234567890;\n",
        "protocol = 1; publishedid = 1234567890;",
        "/* Workshop */ publishedid = 1234567890;",
    ] {
        fs_err::write("meta.cpp", meta).expect("published ID");
        let report = publish::execute(&command(true)).expect("existing item dry run");
        assert!(!report.failed());
        assert_eq!(
            fs_err::read_to_string("meta.cpp").expect("unchanged ID"),
            meta
        );
    }
}

#[sealed_test]
fn invalid_metadata_stops_normal_publish_before_build_and_keeps_previous_release() {
    let _project = project();
    fs_err::create_dir_all(".hemttout/release").expect("previous release");
    fs_err::write(".hemttout/release/keep.txt", "previous build").expect("previous artifact");
    fs_err::write(
        ".hemtt/publish.toml",
        "title = ' '\npreview = 'missing.png'\nrequired_items = ['0']",
    )
    .expect("invalid metadata");
    let error = match publish::execute(&command(false)) {
        Ok(_) => panic!("invalid metadata must stop publishing"),
        Err(error) => error.to_string(),
    };
    for field in ["title", "missing.png", "required_items"] {
        assert!(error.contains(field), "missing {field}: {error}");
    }
    assert_eq!(
        fs_err::read_to_string(".hemttout/release/keep.txt").expect("preserved artifact"),
        "previous build"
    );
    assert!(!Path::new("meta.cpp").exists());
}

#[sealed_test]
fn malformed_existing_id_does_not_start_creation_or_build() {
    let _project = project();
    for value in ["0", "18446744073709551616", "invalid"] {
        let meta = format!("publishedid = {value};\n");
        fs_err::write("meta.cpp", &meta).expect("invalid ID");
        assert!(publish::execute(&command(false)).is_err());
        assert_eq!(
            fs_err::read_to_string("meta.cpp").expect("unchanged invalid ID"),
            meta
        );
        assert!(!Path::new(".hemttout/release").exists());
    }
}

#[sealed_test]
fn failed_release_prevents_steam_initialization() {
    let _project = project();
    fs_err::write("addons/main/config.cpp", "class CfgPatches {").expect("invalid addon");
    let report = publish::execute(&command(false)).expect("build diagnostics");
    assert!(report.failed(), "invalid addon must stop the upload");
    assert!(!Path::new("meta.cpp").exists());
}
