use std::ops::Range;

use hemtt_common::reporting::{Code, Diagnostic, Processed, Severity};

use crate::Expression;

pub struct StrFormat {
    span: Range<usize>,
    expr: Expression,

    diagnostic: Option<Diagnostic>,
}

impl Code for StrFormat {
    fn ident(&self) -> &'static str {
        "SAA4"
    }

    fn severity(&self) -> Severity {
        Severity::Help
    }

    fn message(&self) -> String {
        String::from("using `format [\"%1\", ...]` is slower than using `str ...`")
    }

    fn label_message(&self) -> String {
        format!("use `str {}`", self.expr.source())
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "str {}",
            if matches!(
                self.expr,
                Expression::UnaryCommand(_, _, _) | Expression::BinaryCommand(_, _, _, _)
            ) {
                format!("({})", self.expr.source())
            } else {
                self.expr.source()
            }
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl StrFormat {
    #[must_use]
    pub fn new(span: Range<usize>, expr: Expression, processed: &Processed) -> Option<Self> {
        Self {
            span,
            expr,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Option<Self> {
        let map = processed
            .mapping(self.span.start)
            .expect("span not in mapping");
        if map.was_macro() {
            // Don't emit for WARNING_1 and such macros
            return None;
        }
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        Some(self)
    }
}
