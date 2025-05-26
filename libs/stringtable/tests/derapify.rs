#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use hemtt_stringtable::{Project, derapify, rapify::rapify};
use hemtt_workspace::WorkspacePath;

#[test]
fn derapify_pass() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/rapify/pass.xml").unwrap()).unwrap();
    // Has 2 languages with unique translations
    let mut buffer = Vec::new();
    rapify(&stringtable).unwrap().write(&mut buffer).unwrap();
    let derap =
        derapify("pass".to_owned(), &mut Cursor::new(buffer)).expect("derapify should succeed");
    let mut out = String::new();
    derap
        .to_writer(&mut out, true)
        .expect("to_writer should succeed");
    insta::assert_snapshot!(out);
}

#[test]
fn derapify_containers() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/rapify/containers.xml").unwrap()).unwrap();
    // Has 2 languages with unique translations
    let mut buffer = Vec::new();
    rapify(&stringtable).unwrap().write(&mut buffer).unwrap();
    let derap = derapify("containers".to_owned(), &mut Cursor::new(buffer))
        .expect("derapify should succeed");
    let mut out = String::new();
    derap
        .to_writer(&mut out, true)
        .expect("to_writer should succeed");
    insta::assert_snapshot!(out);
}
