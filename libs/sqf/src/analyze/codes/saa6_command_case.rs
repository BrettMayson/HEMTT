use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

pub struct CommandCase {
    span: Range<usize>,
    used: String,
    wiki: String,

    diagnostic: Option<Diagnostic>,
}

impl Code for CommandCase {
    fn ident(&self) -> &'static str {
        "SAA5"
    }

    fn severity(&self) -> Severity {
        Severity::Help
    }

    fn message(&self) -> String {
        format!("`{}` does not match the wiki's case", self.used)
    }

    fn label_message(&self) -> String {
        "non-standard command case".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("\"{}\"", self.wiki))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CommandCase {
    #[must_use]
    pub fn new(span: Range<usize>, used: String, wiki: String, processed: &Processed) -> Self {
        Self {
            span,
            used,
            wiki,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
