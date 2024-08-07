use std::ops::Range;

use hemtt_workspace::reporting::{diagnostic::Yellow, Code, Diagnostic, Processed};

pub struct MissingSemicolon {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for MissingSemicolon {
    fn ident(&self) -> &'static str {
        "CE4"
    }

    fn message(&self) -> String {
        "property is missing a semicolon".to_string()
    }

    fn label_message(&self) -> String {
        "missing semicolon".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "add a semicolon {} to the end of the property",
            Yellow.paint(";")
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl MissingSemicolon {
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let haystack = &processed.as_str()[self.span.clone()];
        let possible_end = self.span.start
            + haystack
                .find('\n')
                .unwrap_or_else(|| haystack.rfind(|c: char| c != ' ' && c != '}').unwrap_or(0) + 1);
        self.diagnostic =
            Diagnostic::new_for_processed(&self, possible_end..possible_end, processed);
        self
    }
}
