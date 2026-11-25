#![allow(clippy::unwrap_used)]

use hemtt_stringtable::{Project, analyze::lint_all};
use hemtt_workspace::{
    LayerType,
    addons::Addon,
    position::{LineCol, Position},
};

fn read_list(name: &str) -> Vec<String> {
    fs_err::read_to_string(format!(".hemttout/{name}_stringtables.txt")).map_or_else(
        |_| Vec::new(),
        |s| {
            s.lines()
                .map(|l| l.split(" - ").next().unwrap().to_string())
                .collect()
        },
    )
}

#[test]
fn usage_unused_and_missing() {
    let folder = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/usage"));
    let _directory = hemtt_test::directory::TemporaryDirectory::new();
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("stringtable.xml").unwrap();
    let stringtable = Project::read(source.clone()).unwrap();

    let addon = Addon::test_addon();
    {
        let usages = addon.build_data().localizations();
        let mut usages = usages.lock().unwrap();
        let pos = || Position::new(LineCol(0, (1, 0)), LineCol(0, (1, 0)), source.clone());
        for key in [
            "str_test_used",
            "str_test_mixed_case",
            "str_test_format_%1",
            "str_test_missing",
            "str_test_missing_format_%1",
            // used twice, only reported once
            "str_test_used",
        ] {
            usages.push((key.to_string(), pos()));
        }
    }

    let codes = lint_all(&vec![stringtable], None, vec![addon]);

    let idents = codes.iter().map(|c| c.ident()).collect::<Vec<_>>();
    assert!(idents.contains(&"L-L02U"), "{idents:?}");
    assert!(idents.contains(&"L-L02M"), "{idents:?}");
    assert!(!idents.contains(&"L-L02D"), "{idents:?}");

    assert_eq!(
        read_list("unused"),
        ["str_test_also_unused", "str_test_unused"]
    );
    let mut missing = read_list("missing");
    missing.sort();
    assert_eq!(missing, ["str_test_missing", "str_test_missing_format_%1"]);
}
