use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed};

pub struct UnparseableSyntax {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for UnparseableSyntax {
    fn ident(&self) -> &'static str {
        "SPE2"
    }

    fn message(&self) -> String {
        "SQF Syntax could not be parsed".to_string()
    }

    fn label_message(&self) -> String {
        "unparseable syntax".to_string()
    }

    fn note(&self) -> Option<String> {
        Some("HEMTT was not able to determine the structure of this SQF code block.\nThis is likely due to a missing or extra bracket, brace, or parenthesis.\nIt can also occur if a command is not called correctly.".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl UnparseableSyntax {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
