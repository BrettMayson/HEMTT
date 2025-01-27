#![allow(clippy::unwrap_used)]

use hemtt_stringtable::Project;
use hemtt_workspace::WorkspacePath;

#[test]
fn totals_ace_arsenal() {
    let stringtable =
        Project::read(WorkspacePath::slim_file("tests/ace_arsenal.xml").unwrap()).unwrap();
    insta::assert_debug_snapshot!(stringtable);

    assert_eq!(stringtable.name(), "ACE");

    let arsenal = stringtable.packages().first().unwrap();

    assert_eq!(arsenal.name(), "Arsenal");
    insta::assert_debug_snapshot!(arsenal.totals());
}
