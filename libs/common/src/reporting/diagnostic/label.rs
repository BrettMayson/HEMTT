use std::ops::Range;

use codespan_reporting::diagnostic::LabelStyle;

use crate::workspace::WorkspacePath;

#[derive(Debug, Clone)]
pub struct Label {
    style: LabelStyle,
    message: Option<String>,
    file: WorkspacePath,
    span: Range<usize>,
}

impl Label {
    #[must_use]
    pub const fn primary(file: WorkspacePath, span: Range<usize>) -> Self {
        Self {
            style: LabelStyle::Primary,
            message: None,
            file,
            span,
        }
    }

    #[must_use]
    pub const fn secondary(file: WorkspacePath, span: Range<usize>) -> Self {
        Self {
            style: LabelStyle::Secondary,
            message: None,
            file,
            span,
        }
    }

    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    #[must_use]
    pub fn to_codespan(&self) -> codespan_reporting::diagnostic::Label<&WorkspacePath> {
        let mut label =
            codespan_reporting::diagnostic::Label::new(self.style, &self.file, self.span.clone());
        if let Some(message) = &self.message {
            label = label.with_message(message.clone());
        }
        label
    }
}
