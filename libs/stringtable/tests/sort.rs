#![allow(clippy::unwrap_used)]

use std::io::BufReader;

use hemtt_stringtable::Project;

#[test]
fn sort_ace_arsenal() {
    let mut stringtable: Project = quick_xml::de::from_str(
        std::fs::read_to_string("tests/ace_arsenal.xml")
            .unwrap()
            .as_str(),
    )
    .unwrap();
    insta::assert_debug_snapshot!(stringtable);

    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);
}

#[test]
fn sort_comments() {
    let mut stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/sort/comments.xml").unwrap(),
    ))
    .unwrap();
    insta::assert_debug_snapshot!(stringtable);

    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}

#[test]
fn sort_gh822() {
    let mut stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/sort/gh822.xml").unwrap(),
    ))
    .unwrap();
    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}

#[test]
fn sort_containers() {
    let mut stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/sort/containers.xml").unwrap(),
    ))
    .unwrap();
    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}
