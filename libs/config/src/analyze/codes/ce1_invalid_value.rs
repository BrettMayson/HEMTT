use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed};

pub struct InvalidValue {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for InvalidValue {
    fn ident(&self) -> &'static str {
        "CE1"
    }

    fn message(&self) -> String {
        "property's value could not be parsed.".to_string()
    }

    fn label_message(&self) -> String {
        "invalid value".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("use quotes `\"` around the value".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl InvalidValue {
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
