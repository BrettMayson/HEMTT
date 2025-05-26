#![allow(clippy::unwrap_used)]

use hemtt_stringtable::{Project, rapify::rapify};
use hemtt_workspace::WorkspacePath;

#[test]
fn rapify_pass() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/rapify/pass.xml").unwrap()).unwrap();
    // Has 2 languages with unique translations
    let rapify = rapify(&stringtable);
    assert!(rapify.is_some());
    insta::assert_debug_snapshot!(rapify.unwrap());
}

#[test]
fn rapify_containers() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/rapify/containers.xml").unwrap()).unwrap();
    // Has 2 languages with unique translations
    let rapify = rapify(&stringtable);
    assert!(rapify.is_some());
    insta::assert_debug_snapshot!(rapify.unwrap());
}

#[test]
fn rapify_invalid() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/rapify/invalid.xml").unwrap()).unwrap();
    // Cannot be binnerized
    let rapify = rapify(&stringtable);
    assert!(rapify.is_none());
}

#[test]
fn rapify_unescaped() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/rapify/unescaped.xml").unwrap()).unwrap();
    // Cannot be binnerized
    let rapify = rapify(&stringtable);
    assert!(rapify.is_none());
}
