#![allow(clippy::unwrap_used)]

use hemtt_stringtable::{
    Project,
    analyze::{lint_all, lint_one},
};
use hemtt_workspace::{
    LayerType,
    reporting::{Codes, WorkspaceFiles},
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
lint!(l03_no_newlines_in_tags);

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
