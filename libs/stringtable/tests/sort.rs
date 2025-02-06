#![allow(clippy::unwrap_used)]

use hemtt_stringtable::Project;
use hemtt_workspace::WorkspacePath;

#[test]
fn sort_ace_arsenal() {
    let mut stringtable =
        Project::read(WorkspacePath::slim_file("tests/ace_arsenal.xml").unwrap()).unwrap();
    insta::assert_debug_snapshot!(stringtable);

    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);
}

#[test]
fn sort_comments() {
    let mut stringtable =
        Project::read(WorkspacePath::slim_file("tests/sort/comments.xml").unwrap()).unwrap();
    insta::assert_debug_snapshot!(stringtable);

    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}

#[test]
fn sort_gh822() {
    let mut stringtable =
        Project::read(WorkspacePath::slim_file("tests/sort/gh822.xml").unwrap()).unwrap();
    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}

#[test]
fn sort_containers() {
    let mut stringtable =
        Project::read(WorkspacePath::slim_file("tests/sort/containers.xml").unwrap()).unwrap();
    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}
