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
use hemtt_project::ProjectConfig;
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
) -> Result<ConfigReport, Vec<String>> {
    let (config, errors) = parse::config().parse_recovery(processed.as_string());
    config.map_or_else(
        || {
            Err(errors
                .iter()
                .map(|e| chumsky_to_ariadne(processed, e))
                .collect())
        },
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

fn chumsky_to_ariadne(processed: &Processed, err: &Simple<char>) -> String {
    let map = processed.mapping(err.span().start);
    let Some(map) = map else {
        return format!("{:?}: {}", err.span(), err);
    };
    let file = processed.source(map.source()).unwrap();
    let file = file.0.clone();
    let mut out = Vec::new();
    Report::build(
        ariadne::ReportKind::Error,
        file.clone(),
        map.original_column(),
    )
    .with_message(err.to_string())
    .with_label(
        Label::new((
            file,
            map.original_column()..(map.original_column() + err.span().len()),
        ))
        .with_message(err.label().unwrap_or_default()),
    )
    .finish()
    .write_for_stdout(sources(processed.sources()), &mut out)
    .unwrap();
    String::from_utf8(out).unwrap()
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
