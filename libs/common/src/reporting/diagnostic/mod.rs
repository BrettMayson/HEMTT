pub use ansi_term::Colour::*;

use codespan_reporting::{
    diagnostic::{LabelStyle, Severity},
    files::Files,
    term::termcolor::Ansi,
};

use crate::workspace::WorkspacePath;

use self::annotation::Annotation;
pub use self::label::Label;

use super::{Code, WorkspaceFiles};

mod annotation;
mod label;

#[derive(Debug, Clone)]
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

    pub fn new_for_processed(
        code: &impl Code,
        span: std::ops::Range<usize>,
        processed: &crate::reporting::Processed,
    ) -> Option<Self> {
        let mut diag = Self::new(code.ident(), code.message()).set_severity(code.severity());
        let Some(map_start) = processed.mapping(span.start) else {
            return None;
        };
        let Some(map_end) = processed.mapping(span.end) else {
            return None;
        };
        let Some(map_file) = processed.source(map_start.source()) else {
            return None;
        };
        diag.labels.push(
            Label::primary(
                map_file.0.clone(),
                map_start.original_column()..map_end.original_column(),
            )
            .with_message(code.label_message()),
        );
        if let Some(note) = code.note() {
            diag.notes.push(note);
        }
        if let Some(help) = code.help() {
            diag.help.push(help);
        }
        if let Some(suggestion) = code.suggestion() {
            diag.suggestions.push(suggestion);
        }
        Some(diag)
    }

    pub fn simple(code: &impl Code) -> Self {
        let mut diag = Self::new(code.ident(), code.message()).set_severity(code.severity());
        if let Some(note) = code.note() {
            diag.notes.push(note);
        }
        if let Some(help) = code.help() {
            diag.help.push(help);
        }
        if let Some(suggestion) = code.suggestion() {
            diag.suggestions.push(suggestion);
        }
        diag
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
                    .map(|n| format!("{}: {}", Cyan.paint("note"), n.replace('\n', "\n      ")))
                    .collect::<Vec<_>>();
                notes.extend(
                    self.help.iter().map(|h| {
                        format!("{}: {}", Yellow.paint("help"), h.replace('\n', "\n      "))
                    }),
                );
                notes.extend(
                    self.suggestions
                        .iter()
                        .map(|s| format!("{}: {}", Green.paint("try"), s.replace('\n', "\n     "))),
                );
                notes
            })
    }

    #[must_use]
    /// Convert the diagnostic to a string
    ///
    /// # Panics
    /// Will panic if the codespan term writer fails, or produces invalid utf8
    pub fn to_string(&self, files: &WorkspaceFiles) -> String {
        let diag = self.to_codespan();
        let config = codespan_reporting::term::Config::default();
        let mut buffer: Ansi<Vec<u8>> = Ansi::new(Vec::new());
        codespan_reporting::term::emit(&mut buffer, &config, files, &diag)
            .expect("emit should succeed");
        String::from_utf8(buffer.into_inner())
            .expect("utf8")
            .replace("\u{1b}[34m", "\u{1b}[36m")
    }

    #[must_use]
    /// Convert the diagnostic to an annotation for GitHub
    ///
    /// # Panics
    /// Will panic if the file is not found in the workspace
    pub fn to_annotations(&self, files: &WorkspaceFiles) -> Vec<Annotation> {
        self.labels
            .iter()
            .filter_map(|l| {
                if l.style == LabelStyle::Secondary {
                    return None;
                }
                l.message.as_ref()?;
                let start_line_index = files.line_index(&l.file, l.span.start).unwrap();
                let end_line_index = files.line_index(&l.file, l.span.end).unwrap();
                Some(Annotation {
                    path: l.file.path.as_str().to_string(),
                    start_line: files.line_number(&l.file, start_line_index).unwrap(),
                    end_line: files.line_number(&l.file, end_line_index).unwrap(),
                    start_column: files
                        .column_number(&l.file, start_line_index, l.span.start)
                        .unwrap(),
                    end_column: files
                        .column_number(&l.file, end_line_index, l.span.end)
                        .unwrap(),
                    level: self.severity.into(),
                    message: l.message.clone().unwrap(),
                    title: self.message.clone(),
                })
            })
            .collect()
    }
}
