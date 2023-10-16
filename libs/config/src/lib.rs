#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

mod analyze;
mod error;
mod model;

use analyze::Analyze;
use ariadne::{sources, Label, Report};
use chumsky::{prelude::Simple, Parser};
use hemtt_common::reporting::{Code, Processed};

pub use error::Error;
use hemtt_common::project::ProjectConfig;
pub use model::*;
pub mod parse;
pub mod rapify;

/// Parse a config file
///
/// # Errors
/// If the file is invalid
pub fn parse(
    project: Option<&ProjectConfig>,
    processed: &Processed,
) -> Result<ConfigReport, Vec<ChumskyCode>> {
    let (config, errors) = parse::config().parse_recovery(processed.as_string());
    config.map_or_else(
        || Err(errors.iter().map(std::convert::Into::into).collect()),
        |config| {
            Ok(ConfigReport {
                valid: config.valid(project),
                warnings: config.warnings(project, processed),
                errors: config.errors(project, processed),
                config,
            })
        },
    )
}

/// A parsed config file with warnings and errors
pub struct ConfigReport {
    config: Config,
    valid: bool,
    warnings: Vec<Box<dyn Code>>,
    errors: Vec<Box<dyn Code>>,
}

impl ConfigReport {
    #[must_use]
    /// Get the config
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    /// Get the validity
    pub const fn valid(&self) -> bool {
        self.valid
    }

    #[must_use]
    /// Get the warnings
    pub fn warnings(&self) -> &[Box<dyn Code>] {
        &self.warnings
    }

    #[must_use]
    /// Get the errors
    pub fn errors(&self) -> &[Box<dyn Code>] {
        &self.errors
    }
}

#[derive(Debug, Clone)]
/// A chumsky error
pub struct ChumskyCode {
    err: Simple<char>,
}

impl Code for ChumskyCode {
    fn ident(&self) -> &'static str {
        "CHU"
    }

    fn message(&self) -> String {
        self.err.to_string()
    }

    fn label_message(&self) -> String {
        self.err.to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn report_generate_processed(&self, processed: &Processed) -> Option<String> {
        let map = processed.mapping(self.err.span().start);
        let Some(map) = map else {
            return Some(format!("{:?}: {}", self.err.span(), self.err));
        };
        let file = processed.source(map.source()).unwrap();
        let file = file.0.clone();
        let mut out = Vec::new();
        Report::build(
            ariadne::ReportKind::Error,
            file.clone(),
            map.original_column(),
        )
        .with_message(self.err.to_string())
        .with_label(
            Label::new((
                file,
                map.original_column()..(map.original_column() + self.err.span().len()),
            ))
            .with_message(self.err.label().unwrap_or_default()),
        )
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }
}

impl From<ChumskyCode> for Box<dyn Code> {
    fn from(val: ChumskyCode) -> Self {
        Box::new(val)
    }
}

impl From<&Simple<char>> for ChumskyCode {
    fn from(err: &Simple<char>) -> Self {
        Self { err: err.clone() }
    }
}
