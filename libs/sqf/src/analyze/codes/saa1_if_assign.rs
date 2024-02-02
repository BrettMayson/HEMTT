use std::ops::Range;

use hemtt_common::reporting::{Code, Diagnostic, Processed, Severity};

pub struct IfAssign {
    if_cmd: Range<usize>,
    condition: (String, Range<usize>),
    lhs: ((String, bool), Range<usize>),
    rhs: ((String, bool), Range<usize>),

    diagnostic: Option<Diagnostic>,
}

impl Code for IfAssign {
    fn ident(&self) -> &'static str {
        "SAA1"
    }

    fn severity(&self) -> Severity {
        Severity::Help
    }

    fn message(&self) -> String {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            String::from("assignment to if can be replaced with parseNumber")
        } else {
            String::from("assignment to if can be replaced with select")
        }
    }

    fn label_message(&self) -> String {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            String::from("use parseNumber")
        } else {
            String::from("use select")
        }
    }

    fn suggestion(&self) -> Option<String> {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            Some(format!("parseNumber {}", self.condition.0.as_str(),))
        } else {
            Some(format!(
                "[{}, {}] select ({})",
                if self.rhs.0 .1 {
                    format!("\"{}\"", self.rhs.0 .0.as_str())
                } else {
                    self.rhs.0 .0.clone()
                },
                if self.lhs.0 .1 {
                    format!("\"{}\"", self.lhs.0 .0.as_str())
                } else {
                    self.lhs.0 .0.clone()
                },
                self.condition.0.as_str(),
            ))
        }
    }

    fn note(&self) -> Option<String> {
        Some(
            if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
                "parseNumber returns 1 for true and 0 for false"
            } else {
                "the if and else blocks only return constant values\nselect is faster in this case"
            }
            .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl IfAssign {
    #[must_use]
    pub fn new(
        if_cmd: Range<usize>,
        condition: (String, Range<usize>),
        lhs: ((String, bool), Range<usize>),
        rhs: ((String, bool), Range<usize>),
        processed: &Processed,
    ) -> Self {
        Self {
            if_cmd,
            condition,
            lhs,
            rhs,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let haystack = &processed.as_str()[self.rhs.1.end..];
        let end_position = self.rhs.1.end + haystack.find(|c: char| c == '}').unwrap_or(0) + 1;
        self.diagnostic =
            Diagnostic::new_for_processed(&self, self.if_cmd.start..end_position, processed);
        self
    }
}
