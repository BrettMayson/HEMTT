#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

use std::sync::Arc;

mod analyze;
mod model;
pub mod parse;
pub mod rapify;
pub use model::*;

pub use analyze::CONFIG_LINTS;

use analyze::{Analyze, CfgPatch, ChumskyCode};
use chumsky::Parser;
use hemtt_common::version::Version;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    lint::LintManager,
    reporting::{Code, Codes, Processed, Severity},
};

#[must_use]
pub fn lint_check(project: &ProjectConfig) -> Codes {
    let mut manager = LintManager::new(project.lints().config().clone(), ());
    if let Err(e) = manager.extend(
        CONFIG_LINTS
            .iter()
            .map(|l| (**l).clone())
            .collect::<Vec<_>>(),
    ) {
        return e;
    }
    vec![]
}

/// Parse a config file
///
/// # Errors
/// If the file is invalid
pub fn parse(
    project: Option<&ProjectConfig>,
    processed: &Processed,
) -> Result<ConfigReport, Codes> {
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
            let mut manager = LintManager::new(
                project.map_or_else(Default::default, |project| project.lints().config().clone()),
                (),
            );
            manager.extend(
                CONFIG_LINTS
                    .iter()
                    .map(|l| (**l).clone())
                    .collect::<Vec<_>>(),
            )?;
            Ok(ConfigReport {
                codes: config.analyze(project, processed, &manager),
                patches: config.get_patches(),
                config,
            })
        },
    )
}

/// A parsed config file with warnings and errors
pub struct ConfigReport {
    config: Config,
    codes: Codes,
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
    /// Get the codes
    pub fn codes(&self) -> &[Arc<dyn Code>] {
        &self.codes
    }

    #[must_use]
    /// Get the warnings
    pub fn warnings(&self) -> Vec<&Arc<dyn Code>> {
        self.codes
            .iter()
            .filter(|c| c.severity() == Severity::Warning)
            .collect::<Vec<_>>()
    }

    #[must_use]
    /// Get the errors
    pub fn errors(&self) -> Vec<&Arc<dyn Code>> {
        self.codes
            .iter()
            .filter(|c| c.severity() == Severity::Error)
            .collect::<Vec<_>>()
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
