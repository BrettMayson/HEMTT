use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed};

pub struct InvalidToken {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for InvalidToken {
    fn ident(&self) -> &'static str {
        "SPE1"
    }

    fn message(&self) -> String {
        "invalid token".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl InvalidToken {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
