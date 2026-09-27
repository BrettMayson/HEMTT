#![allow(clippy::unwrap_used)]

//! Which files get config checked. Both the CLI and the language server rely
//! on this answer, so a change here changes what the editor shows.

use hemtt_config::files::{can_rapify, checkable};
use hemtt_workspace::{LayerType, addons::Addon};

const ROOT: &str = "tests/files";

fn workspace() -> hemtt_workspace::WorkspacePath {
    hemtt_workspace::Workspace::builder()
        .physical(&std::path::PathBuf::from(ROOT), LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap()
}

fn addon() -> Addon {
    Addon::new(
        &std::path::PathBuf::from(ROOT),
        "main".to_string(),
        hemtt_workspace::addons::Location::Addons,
    )
    .unwrap()
}

fn names(mut files: Vec<hemtt_workspace::WorkspacePath>) -> Vec<String> {
    files.sort_by_key(hemtt_workspace::WorkspacePath::filename);
    files
        .iter()
        .map(hemtt_workspace::WorkspacePath::filename)
        .collect()
}

#[test]
fn rapifiable_extensions() {
    let workspace = workspace();
    for name in ["config.cpp", "checked.rvmat", "mission.ext"] {
        let path = workspace.join(format!("/addons/main/{name}")).unwrap();
        assert!(can_rapify(&path).unwrap(), "{name} should be rapifiable");
    }
    let path = workspace.join("/addons/main/notes.txt").unwrap();
    assert!(!can_rapify(&path).unwrap(), "notes.txt is not a config");
}

/// The editor used to check only `config.cpp`, so an error in a `.rvmat` or
/// `.ext` was reported by `hemtt check` and never shown.
#[test]
fn checks_every_rapifiable_file() {
    let files = names(checkable(&workspace(), &addon()).unwrap());
    assert!(files.contains(&"config.cpp".to_string()), "{files:?}");
    assert!(files.contains(&"checked.rvmat".to_string()), "{files:?}");
    assert!(files.contains(&"mission.ext".to_string()), "{files:?}");
    assert!(!files.contains(&"notes.txt".to_string()), "{files:?}");
}

/// The editor used to ignore `rapify.exclude`, so it reported diagnostics on
/// files the project had deliberately excluded.
#[test]
fn honours_exclude() {
    let files = names(checkable(&workspace(), &addon()).unwrap());
    assert!(!files.contains(&"skipped.rvmat".to_string()), "{files:?}");
}
