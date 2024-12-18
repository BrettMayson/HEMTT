#![allow(clippy::unwrap_used)]

use std::io::BufReader;

use hemtt_stringtable::Project;

#[test]
fn sort() {
    let mut stringtable = Project::from_reader(BufReader::new(
        std::fs::File::open("tests/comments.xml").unwrap(),
    ))
    .unwrap();
    insta::assert_debug_snapshot!(stringtable);

    stringtable.sort();

    insta::assert_debug_snapshot!(stringtable);

    let mut out = String::new();
    stringtable.to_writer(&mut out).unwrap();

    insta::assert_snapshot!(out);
}
