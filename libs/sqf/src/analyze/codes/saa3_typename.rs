use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

pub struct Typename {
    span: Range<usize>,
    constant: (String, Range<usize>, usize),

    diagnostic: Option<Diagnostic>,
}

impl Code for Typename {
    fn ident(&self) -> &'static str {
        "SAA3"
    }

    fn severity(&self) -> Severity {
        Severity::Help
    }

    fn message(&self) -> String {
        String::from("using `typeName` on a constant is slower than using the type directly")
    }

    fn label_message(&self) -> String {
        "`typeName` on a constant".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("\"{}\"", self.constant.0))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Typename {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        constant: (String, Range<usize>, usize),
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            constant,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
