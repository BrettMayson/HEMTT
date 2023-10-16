//! Reporting module

use std::fmt::{Debug, Display};

mod error;
mod output;
mod processed;
mod symbol;
mod token;
mod whitespace;

pub use error::Error;
pub use output::Output;
pub use processed::{Mapping, Processed};
pub use symbol::Symbol;
pub use token::Token;
pub use whitespace::Whitespace;

use crate::position::Position;

/// A coded error or warning
pub trait Code: Send + Sync {
    /// The token of the code
    fn token(&self) -> Option<&Token> {
        None
    }
    /// The code identifier
    fn ident(&self) -> &'static str;
    /// Message explaining the error
    fn message(&self) -> String;
    /// Message explaining the error, applied to the label
    fn label_message(&self) -> String;
    /// Help message, if any
    fn help(&self) -> Option<String>;

    /// A report for the CLI
    fn report_generate(&self) -> Option<String> {
        None
    }
    /// A report for the CLI, applied to the processed file
    fn report_generate_processed(&self, _processed: &Processed) -> Option<String> {
        None
    }

    /// A report for CI
    fn ci_generate(&self) -> Vec<Annotation> {
        Vec::new()
    }
    /// A report for CI, applied to the processed file
    fn ci_generate_processed(&self, _processed: &Processed) -> Vec<Annotation> {
        Vec::new()
    }
    /// Helper to generate an annotation for CI
    fn annotation(&self, level: AnnotationLevel, path: String, span: &Position) -> Annotation {
        Annotation {
            path,
            start_line: span.start().1 .0,
            end_line: span.end().1 .0,
            start_column: span.start().1 .1,
            end_column: span.end().1 .1,
            level,
            message: self.message(),
            title: self.label_message(),
        }
    }

    #[cfg(feature = "lsp")]
    /// Generate a diagnostic for the LSP
    fn lsp_generate(&self) -> Option<(VfsPath, Diagnostic)> {
        None
    }
    #[cfg(feature = "lsp")]
    /// Generate a diagnostic for the LSP, applied to the processed file
    fn lsp_generate_processed(&self, _processed: &Processed) -> Vec<(VfsPath, Diagnostic)> {
        Vec::new()
    }
    #[cfg(feature = "lsp")]
    /// Helper to generate a diagnostic for the LSP
    fn lsp_diagnostic(&self, range: Range) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(lsp_types::NumberOrString::String(self.ident().to_string())),
            code_description: None,
            source: Some(String::from("HEMTT")),
            message: self.label_message(),
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

impl Debug for dyn Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Annotation level
pub enum AnnotationLevel {
    /// Annotate a notice
    Notice,
    /// Annotate a warning
    Warning,
    /// Annotate an error
    Error,
}

impl Display for AnnotationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notice => write!(f, "notice"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Annotation for a CI environment
///
/// <https://github.com/actions/toolkit/tree/main/packages/core#annotations>
pub struct Annotation {
    /// The path of the file to annotate
    pub path: String,
    /// The start line of the annotation
    pub start_line: usize,
    /// The end line of the annotation
    pub end_line: usize,
    /// The start column of the annotation
    pub start_column: usize,
    /// The end column of the annotation
    pub end_column: usize,
    /// The annotation level
    pub level: AnnotationLevel,
    /// The annotation message
    pub message: String,
    /// The annotation title
    pub title: String,
}

impl Annotation {
    #[must_use]
    /// Generate a line for the CI annotation
    pub fn line(&self) -> String {
        format!(
            "{}||{}||{}||{}||{}||{}||{}||{}\n",
            self.start_line,
            self.end_line,
            self.start_column,
            self.end_column,
            self.level,
            self.title,
            self.message,
            self.path,
        )
    }
}
