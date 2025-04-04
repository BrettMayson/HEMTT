#![allow(clippy::unwrap_used)]

use hemtt_stringtable::{Project, rapify::rapify};
use hemtt_workspace::WorkspacePath;

#[test]
fn bin_pass() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/bin/pass.xml").unwrap()).unwrap();
    // Has 2 languages with unique translations
    let bin = rapify(&stringtable);
    assert!(bin.is_some());
    insta::assert_debug_snapshot!(bin.unwrap());
}

#[test]
fn bin_containers() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/bin/containers.xml").unwrap()).unwrap();
    // Has 2 languages with unique translations
    let bin = rapify(&stringtable);
    assert!(bin.is_some());
    insta::assert_debug_snapshot!(bin.unwrap());
}

#[test]
fn bin_invalid() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/bin/invalid.xml").unwrap()).unwrap();
    // Cannot be binnerized
    let bin = rapify(&stringtable);
    assert!(bin.is_none());
}

#[test]
fn bin_unescaped() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/bin/unescaped.xml").unwrap()).unwrap();
    // Cannot be binnerized
    let bin = rapify(&stringtable);
    assert!(bin.is_none());
}
