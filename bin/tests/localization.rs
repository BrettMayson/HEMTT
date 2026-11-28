#![allow(clippy::unwrap_used)]

//! `hemtt localization` end to end.

mod common;

use common::{fixture, read, run, write};
use hemtt::Error;
use sealed_test::prelude::*;

const UNSORTED: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Project name="TEST">
    <Package name="Main">
        <Key ID="STR_test_zebra">
            <Original>Zebra</Original>
            <German>Zebra</German>
            <English>Zebra</English>
        </Key>
        <Key ID="STR_test_apple">
            <Original>Apple</Original>
            <English>Apple</English>
        </Key>
    </Package>
</Project>
"#;

const PATH: &str = "addons/main/stringtable.xml";

#[sealed_test]
fn sort_orders_keys_and_languages() {
    let _directory = fixture("workspace_simple");
    write(PATH, UNSORTED);
    run(&["localization", "sort"]).succeeds();
    let sorted = read(PATH);
    let apple = sorted.find("STR_test_apple").unwrap();
    let zebra = sorted.find("STR_test_zebra").unwrap();
    assert!(apple < zebra, "keys are sorted\n{sorted}");
    let zebra_key = &sorted[zebra..];
    let original = zebra_key.find("<Original>").unwrap();
    let english = zebra_key.find("<English>").unwrap();
    let german = zebra_key.find("<German>").unwrap();
    assert!(
        original < english && english < german,
        "languages are sorted\n{sorted}"
    );

    // Sorting is stable
    run(&["localization", "sort"]).succeeds();
    assert_eq!(read(PATH), sorted);
}

#[sealed_test]
fn sort_only_lang_keeps_key_order() {
    let _directory = fixture("workspace_simple");
    write(PATH, UNSORTED);
    run(&["localization", "sort", "--only-lang"]).succeeds();
    let sorted = read(PATH);
    assert!(
        sorted.find("STR_test_zebra").unwrap() < sorted.find("STR_test_apple").unwrap(),
        "keys keep their order\n{sorted}"
    );
    let zebra_key = &sorted[sorted.find("STR_test_zebra").unwrap()..];
    assert!(
        zebra_key.find("<English>").unwrap() < zebra_key.find("<German>").unwrap(),
        "languages are sorted\n{sorted}"
    );
}

#[sealed_test]
fn sort_unknown_language_fails_and_keeps_the_file() {
    let _directory = fixture("workspace_simple");
    let content = UNSORTED.replace("<German>Zebra</German>", "<Klingon>Zebra</Klingon>");
    write(PATH, &content);
    let result = run(&["localization", "sort"]);
    assert!(matches!(
        result.fails(),
        Error::StringtableUnknownLanguage(_)
    ));
    assert_eq!(read(PATH), content, "nothing is lost");
}

#[sealed_test]
fn sort_invalid_xml_fails() {
    let _directory = fixture("workspace_simple");
    let content = "<Project name=\"TEST\"><Package name=\"Main\"><Key ID=\"broken\">";
    write(PATH, content);
    let result = run(&["localization", "sort"]);
    assert!(matches!(result.fails(), Error::Stringtable(_)));
    assert_eq!(read(PATH), content, "nothing is written");
}

#[sealed_test]
fn coverage_reports_missing_languages() {
    let _directory = fixture("workspace_simple");
    write(PATH, UNSORTED);
    run(&["localization", "coverage"])
        .succeeds()
        .prints("English")
        .prints("German");
}

#[sealed_test]
fn coverage_json() {
    let _directory = fixture("workspace_simple");
    write(PATH, UNSORTED);
    let result = run(&["localization", "coverage", "--format", "json"]);
    result.succeeds();
    let line = result
        .output
        .lines()
        .find(|l| l.starts_with('['))
        .unwrap_or_else(|| panic!("no json in\n{}", result.output));
    let json: serde_json::Value = serde_json::from_str(line).unwrap();
    assert_eq!(json[2]["language"], "German", "{json}");
    assert_eq!(json[2]["percent"], 50.0, "{json}");
    assert_eq!(json[2]["missing"][0], "addons/main", "{json}");
}

#[sealed_test]
fn coverage_invalid_xml_fails() {
    let _directory = fixture("workspace_simple");
    write(PATH, "<Project name=\"TEST\"><Package");
    let result = run(&["localization", "coverage"]);
    assert!(matches!(result.fails(), Error::Stringtable(_)));
}
