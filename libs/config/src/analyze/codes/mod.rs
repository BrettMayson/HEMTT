use chumsky::error::Simple;
use hemtt_common::reporting::{Code, Diagnostic, Processed};

pub mod ce1_invalid_value;
pub mod ce2_invalid_value_macro;
pub mod ce3_duplicate_property;
pub mod ce4_missing_semicolon;
pub mod ce5_unexpected_array;
pub mod ce6_expected_array;
pub mod ce7_missing_parent;

pub mod cw1_parent_case;
pub mod cw2_magwell_missing_magazine;

#[derive(Debug, Clone)]
/// A chumsky error
pub struct ChumskyCode {
    err: Simple<char>,
    diagnostic: Option<Diagnostic>,
}

impl Code for ChumskyCode {
    fn ident(&self) -> &'static str {
        "CCHU"
    }

    fn message(&self) -> String {
        self.err.to_string()
    }
}

impl ChumskyCode {
    pub fn new(err: Simple<char>, processed: &Processed) -> Self {
        Self {
            err,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.err.span(), processed);
        self
    }
}
