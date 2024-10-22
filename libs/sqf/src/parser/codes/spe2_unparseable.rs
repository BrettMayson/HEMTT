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
        "unparseable syntax".to_string()
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
