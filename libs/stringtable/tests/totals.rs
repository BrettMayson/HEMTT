#![allow(clippy::unwrap_used)]

use hemtt_stringtable::Project;

#[test]
fn totals_ace_arsenal() {
    let stringtable: Project = quick_xml::de::from_str(
        std::fs::read_to_string("tests/ace_arsenal.xml")
            .unwrap()
            .as_str(),
    )
    .unwrap();
    insta::assert_debug_snapshot!(stringtable);

    assert_eq!(stringtable.name(), "ACE");

    let arsenal = stringtable.packages().first().unwrap();

    assert_eq!(arsenal.name(), "Arsenal");
    insta::assert_debug_snapshot!(arsenal.totals());
}
