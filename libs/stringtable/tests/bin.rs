#![allow(clippy::unwrap_used)]

use std::io::BufReader;

use hemtt_stringtable::{rapify::rapify, Project};

#[test]
fn bin_pass() {
    let stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/bin/pass.xml").unwrap(),
    ))
    .unwrap();
    // Has 2 languages with unique translations
    let bin = rapify(&stringtable);
    assert!(bin.is_some());
    insta::assert_debug_snapshot!(bin.unwrap());
}

#[test]
fn bin_containers() {
    let stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/bin/containers.xml").unwrap(),
    ))
    .unwrap();
    // Has 2 languages with unique translations
    let bin = rapify(&stringtable);
    assert!(bin.is_some());
    insta::assert_debug_snapshot!(bin.unwrap());
}

#[test]
fn bin_invalid() {
    let stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/bin/invalid.xml").unwrap(),
    ))
    .unwrap();
    // Cannot be binnerized
    let bin = rapify(&stringtable);
    assert!(bin.is_none());
}

#[test]
fn bin_unescaped() {
    let stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/bin/unescaped.xml").unwrap(),
    ))
    .unwrap();
    // Cannot be binnerized
    let bin = rapify(&stringtable);
    assert!(bin.is_none());
}
