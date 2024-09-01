use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

pub struct FormatArgs {
    span: Range<usize>,
    problem: String,

    diagnostic: Option<Diagnostic>,
}

impl Code for FormatArgs {
    fn ident(&self) -> &'static str {
        "SAA7"
    }

    fn severity(&self) -> Severity {
        Severity::Help
    }

    fn message(&self) -> String {
        self.problem.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl FormatArgs {
    #[must_use]
    pub fn new(span: Range<usize>, problem: String, processed: &Processed) -> Self {
        Self {
            span,
            problem,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
