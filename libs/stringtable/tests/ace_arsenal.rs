#![allow(clippy::unwrap_used)]

use hemtt_stringtable::Project;

#[test]
fn sort() {
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
