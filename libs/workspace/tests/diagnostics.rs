#![allow(clippy::unwrap_used)]

use codespan_reporting::diagnostic::Severity;
use hemtt_workspace::{
    LayerType,
    reporting::{Diagnostic, Label, WorkspaceFiles},
};

const ROOT: &str = "tests/diagnostics/";

#[test]
fn python() {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let diagnostic = Diagnostic::new("T1".to_string(), "using python 2".to_string())
        .set_severity(Severity::Warning)
        .with_label(
            Label::primary(workspace.join("example.py").unwrap(), 0..5)
                .with_message("using outdated `print`".to_string()),
        )
        .with_label(Label::secondary(
            workspace.join("example.py").unwrap(),
            6..19,
        ))
        .with_note("python2 is not supported".to_string())
        .with_help("`print` is replaced by `print()`".to_string())
        .with_suggestion("print()".to_string())
        .to_string(&WorkspaceFiles::new())
        .replace('\r', "");
    let expected = std::fs::read_to_string(folder.join("stderr.ansi"))
        .unwrap()
        .replace('\r', "");
    if expected.is_empty() {
        std::fs::write(folder.join("stderr.ansi"), &diagnostic).unwrap();
    }
    assert_eq!(diagnostic, expected);
}
