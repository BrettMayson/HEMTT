use chumsky::error::Simple;
use hemtt_common::reporting::{Code, Processed};

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
    diagnostic: Option<String>,
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
    pub const fn new(err: Simple<char>, processed: &Processed) -> Self {
        Self {
            err,
            diagnostic: None,
        }
        .report_generate_processed(processed)
    }

    const fn report_generate_processed(mut self, processed: &Processed) -> Self {
        self
        // let map = processed.mapping(self.err.span().start);
        // let Some(map) = map else {
        //     self.diagnostic = Some(format!("{:?}: {}", self.err.span(), self.err));
        //     return self;
        // };
        // let file = processed.source(map.source()).unwrap();
        // let file = file.0.clone();
        // let mut out = Vec::new();
        // Report::build(
        //     ariadne::ReportKind::Error,
        //     file.clone(),
        //     map.original_column(),
        // )
        // .with_message(self.err.to_string())
        // .with_label(
        //     Label::new((
        //         file,
        //         map.original_column()..(map.original_column() + self.err.span().len()),
        //     ))
        //     .with_message(self.err.label().unwrap_or_default()),
        // )
        // .finish()
        // .write_for_stdout(sources(processed.sources()), &mut out)
        // .unwrap();
        // self.diagnostic = Some(String::from_utf8(out).unwrap());
        // self
    }
}
