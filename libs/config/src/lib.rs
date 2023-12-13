#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

mod analyze;
mod error;
mod model;

use std::sync::Arc;

use analyze::{codes::ChumskyCode, Analyze, CfgPatch};
use chumsky::Parser;
use hemtt_common::{
    reporting::{Code, Processed},
    version::Version,
};

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
) -> Result<ConfigReport, Vec<Arc<dyn Code>>> {
    let (config, errors) = parse::config().parse_recovery(processed.as_str());
    config.map_or_else(
        || {
            Err(errors
                .into_iter()
                .map(|e| {
                    let e: Arc<dyn Code> = Arc::new(ChumskyCode::new(e, processed));
                    e
                })
                .collect())
        },
        |config| {
            Ok(ConfigReport {
                warnings: config.warnings(project, processed),
                errors: config.errors(project, processed),
                patches: config.get_patches(),
                config,
            })
        },
    )
}

/// A parsed config file with warnings and errors
pub struct ConfigReport {
    config: Config,
    warnings: Vec<Arc<dyn Code>>,
    errors: Vec<Arc<dyn Code>>,
    patches: Vec<CfgPatch>,
}

impl ConfigReport {
    #[must_use]
    /// Get the config
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    /// Consumes the report and returns the config
    pub fn into_config(self) -> Config {
        self.config
    }

    #[must_use]
    /// Get the warnings
    pub fn warnings(&self) -> &[Arc<dyn Code>] {
        &self.warnings
    }

    #[must_use]
    /// Get the errors
    pub fn errors(&self) -> &[Arc<dyn Code>] {
        &self.errors
    }

    #[must_use]
    /// Get the patches
    pub fn patches(&self) -> &[CfgPatch] {
        &self.patches
    }

    #[must_use]
    /// Get the required version, picking the highest from all patches
    pub fn required_version(&self) -> (Version, Option<CfgPatch>) {
        let mut version = Version::new(0, 0, 0, None);
        let mut patch = None;
        for each in &self.patches {
            if each.required_version() > &version {
                version = each.required_version().clone();
                patch = Some(each.clone());
            }
        }
        (version, patch)
    }
}
