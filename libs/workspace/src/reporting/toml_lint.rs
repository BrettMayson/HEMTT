use codespan_reporting::diagnostic::Severity;
use hemtt_common::toml_lint::TomlLint;
use std::ops::Range;

use crate::{
    WorkspacePath,
    reporting::{Code, Diagnostic, Label},
};

pub struct TomlLintCode {
    span: Range<usize>,
    severity: Severity,
    note: Option<String>,
    help: Option<String>,
    message: Option<String>,
    label: Option<String>,
    diagnostic: Option<Diagnostic>,
}

impl Code for TomlLintCode {
    fn ident(&self) -> &'static str {
        "L-TOML"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn note(&self) -> Option<String> {
        self.note.clone()
    }

    fn help(&self) -> Option<String> {
        self.help.clone()
    }

    fn message(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| "TOML lint violation".to_string())
    }

    fn label_message(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| "matches pattern".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl TomlLintCode {
    pub fn new_file(toml_lint: &TomlLint, span: Range<usize>, path: &WorkspacePath) -> Self {
        let severity = toml_lint.severity().unwrap_or(Severity::Warning);
        let note = toml_lint.note().map(std::string::ToString::to_string);
        let help = toml_lint.help().map(std::string::ToString::to_string);
        let message = toml_lint.message().map(std::string::ToString::to_string);
        let label = toml_lint.label().map(std::string::ToString::to_string);

        Self {
            span,
            severity,
            note,
            help,
            message,
            label,
            diagnostic: None,
        }
        .generate(path)
    }

    fn generate(mut self, path: &WorkspacePath) -> Self {
        let mut diag = Diagnostic::from_code(&self);
        diag.labels.push(
            Label::primary(path.clone(), self.span.clone()).with_message(self.label_message()),
        );
        self.diagnostic = Some(diag);
        self
    }
}
