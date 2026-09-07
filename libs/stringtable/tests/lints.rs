#![allow(clippy::unwrap_used)]

use hemtt_stringtable::{
    Project,
    analyze::{lint_all, lint_one},
};
use hemtt_workspace::{
    LayerType,
    addons::Addon,
    position::{LineCol, Position},
    reporting::{Codes, WorkspaceFiles},
    WorkspacePath,
};

const ROOT: &str = "tests/lints/";

macro_rules! lint {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                insta::assert_snapshot!(lint(stringify!($dir)));
            }
        }
    };
}

lint!(l01_sorted);
lint!(l02_usage_single_quotes);
lint!(l03_no_newlines_in_tags);

#[test]
fn l02_usage_custom_prefix() {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("custom_prefix.xml").unwrap();
    let stringtable = Project::read(source).unwrap();
    let config = hemtt_common::config::ProjectConfig::from_file(
        &folder.join("custom_prefix.toml"),
    )
    .unwrap();
    let addon = Addon::test_addon();
    addon
        .build_data()
        .localizations()
        .lock()
        .unwrap()
        .push((
            "str_ls_common_confirm".to_string(),
            Position::new(
                LineCol(0, (1, 1)),
                LineCol(0, (1, 1)),
                WorkspacePath::slim_file("tests/lints/custom_prefix.xml").unwrap(),
            ),
        ));

    let mut codes = lint_one(&stringtable, Some(&config), vec![addon]);
    codes.retain(|code| code.ident().starts_with("L-L02"));
    assert!(codes.is_empty(), "{codes:?}");
}

fn lint(file: &str) -> String {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.xml")).unwrap();
    let workspace_files = WorkspaceFiles::new();
    let stringtable = Project::read(source).unwrap();

    let mut codes: Codes = Vec::new();
    codes.extend(lint_one(&stringtable, None, vec![]));
    codes.extend(lint_all(&vec![stringtable], None, vec![]));

    codes.retain(|e| {
        e.ident().starts_with(&format!(
            "L-{}",
            file.split_once('_').unwrap().0.to_uppercase()
        ))
    });

    codes
        .iter()
        .map(|e| e.diagnostic().unwrap().to_string(&workspace_files))
        .collect::<Vec<_>>()
        .join("\n")
        .replace('\r', "")
}
