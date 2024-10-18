//! Reporting module

use std::fmt::Debug;

pub mod diagnostic;
mod files;
mod output;
mod processed;
mod symbol;
mod token;
mod whitespace;

pub use codespan_reporting::diagnostic::Severity;
pub use diagnostic::{Diagnostic, Label};
pub use files::{WorkspaceFile, WorkspaceFiles};
pub use output::Output;
pub use processed::{Mapping, Processed, Sources};
pub use symbol::Symbol;
pub use token::Token;
pub use whitespace::Whitespace;

pub type Codes = Vec<std::sync::Arc<dyn Code>>;

pub trait CodesExt {
    fn failed(&self) -> bool;
}

impl CodesExt for Codes {
    fn failed(&self) -> bool {
        self.iter().any(|c| c.severity() == Severity::Error)
    }
}

/// A coded error or warning
pub trait Code: Send + Sync {
    /// The token of the code
    fn token(&self) -> Option<&Token> {
        None
    }
    /// Was the code from an include
    fn include(&self) -> bool {
        false
    }
    /// The code identifier
    fn ident(&self) -> &'static str;
    fn link(&self) -> Option<&str> {
        None
    }
    /// Message explaining the error
    fn message(&self) -> String;
    /// Message explaining the error, applied to the label
    fn label_message(&self) -> String {
        self.message()
    }
    /// Severity of the error
    fn severity(&self) -> Severity {
        Severity::Error
    }
    /// Help message, if any
    fn help(&self) -> Option<String> {
        None
    }
    /// Note, if any
    fn note(&self) -> Option<String> {
        None
    }
    /// Code suggestion, if any
    fn suggestion(&self) -> Option<String> {
        None
    }

    /// A diagnostic for the LSP / terminal
    fn diagnostic(&self) -> Option<Diagnostic> {
        let mut diag = Diagnostic::new(self.ident(), self.message()).set_severity(self.severity());
        if let Some(token) = self.token() {
            diag = diag.with_label(
                Label::primary(token.position().path().clone(), token.position().span())
                    .with_message(self.label_message()),
            );
        }
        if let Some(note) = self.note() {
            diag = diag.with_note(note);
        }
        if let Some(help) = self.help() {
            diag = diag.with_help(help);
        }
        if let Some(suggestion) = self.suggestion() {
            diag = diag.with_suggestion(suggestion);
        }
        diag = self.expand_diagnostic(diag);
        Some(diag)
    }

    /// Expand the default diagnostic with additional information
    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag
    }
}

impl Debug for dyn Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident())
    }
}
