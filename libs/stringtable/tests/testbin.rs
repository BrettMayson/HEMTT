#![allow(clippy::unwrap_used)]

use std::io::BufReader;

use hemtt_stringtable::{rapify::rapify, Project};

#[test]
fn testbin1() {
    let stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/testbin1.xml").unwrap(),
    ))
    .unwrap();
    // Has 2 languages with unique translations
    let bin = rapify(&stringtable);
    assert!(bin.is_some());
    insta::assert_debug_snapshot!(bin.unwrap());
}

#[test]
fn testbin2() {
    let stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/testbin2.xml").unwrap(),
    ))
    .unwrap();
    // Cannot be binnerized
    let bin = rapify(&stringtable);
    assert!(bin.is_none());
}
