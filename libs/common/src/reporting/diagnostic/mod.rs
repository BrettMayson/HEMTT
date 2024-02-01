pub use ansi_term::Colour::*;

use codespan_reporting::{diagnostic::Severity, term::termcolor::Ansi};

use crate::workspace::WorkspacePath;

pub use self::label::Label;

use super::WorkspaceFiles;

mod label;

pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Vec<String>,
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            help: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    #[must_use]
    pub const fn set_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    #[must_use]
    pub fn with_code(mut self, code: String) -> Self {
        self.code = code;
        self
    }

    #[must_use]
    pub fn set_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    #[must_use]
    pub fn with_labels(mut self, labels: Vec<Label>) -> Self {
        self.labels.extend(labels);
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    #[must_use]
    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes.extend(notes);
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    #[must_use]
    pub fn with_helps(mut self, help: Vec<String>) -> Self {
        self.help.extend(help);
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    fn to_codespan(&self) -> codespan_reporting::diagnostic::Diagnostic<&WorkspacePath> {
        codespan_reporting::diagnostic::Diagnostic::new(self.severity)
            .with_code(&self.code)
            .with_message(&self.message)
            .with_labels(self.labels.iter().map(|l| l.to_codespan()).collect())
            .with_notes({
                let mut notes = self
                    .notes
                    .iter()
                    .map(|n| format!("{}: {}", Cyan.paint("note"), n))
                    .collect::<Vec<_>>();
                notes.extend(
                    self.help
                        .iter()
                        .map(|h| format!("{}: {}", Yellow.paint("help"), h)),
                );
                notes.extend(
                    self.suggestions
                        .iter()
                        .map(|s| format!("{}: {}", Green.paint("try"), s)),
                );
                notes
            })
    }

    #[must_use]
    pub fn to_string(&self, files: &WorkspaceFiles) -> String {
        let diag = self.to_codespan();
        let config = codespan_reporting::term::Config::default();
        let mut buffer: Ansi<Vec<u8>> = Ansi::new(Vec::new());
        codespan_reporting::term::emit(&mut buffer, &config, files, &diag).unwrap();
        String::from_utf8(buffer.into_inner()).unwrap()
    }
}
