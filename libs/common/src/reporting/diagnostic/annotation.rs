use std::fmt::Display;

use codespan_reporting::diagnostic::Severity;

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
    pub level: Level,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Annotation level
pub enum Level {
    /// Annotate a notice
    Notice,
    /// Annotate a warning
    Warning,
    /// Annotate an error
    Error,
}

impl From<Severity> for Level {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Warning => Self::Warning,
            Severity::Help | Severity::Note => Self::Notice,
            Severity::Bug | Severity::Error => Self::Error,
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notice => write!(f, "notice"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}
