use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

pub struct FindInStr {
    span: Range<usize>,
    haystack: (String, Range<usize>),
    needle: (String, Range<usize>),

    diagnostic: Option<Diagnostic>,
}

impl Code for FindInStr {
    fn ident(&self) -> &'static str {
        "SAA2"
    }

    fn severity(&self) -> Severity {
        Severity::Help
    }

    fn message(&self) -> String {
        String::from("string search using `in` is faster than `find`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "{} in {}",
            self.needle.0.as_str(),
            self.haystack.0.as_str()
        ))
    }

    fn label_message(&self) -> String {
        String::from("using `find` with -1")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl FindInStr {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        haystack: (String, Range<usize>),
        needle: (String, Range<usize>),
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            haystack,
            needle,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
