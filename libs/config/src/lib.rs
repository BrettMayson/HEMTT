#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

use std::sync::{Arc, Mutex};

pub mod analyze;
pub mod display;
mod model;
pub mod parse;
pub mod rapify;

pub use model::*;

use analyze::{Analyze, CfgPatch, ChumskyCode, LintData};
use chumsky::Parser;
use hemtt_common::version::Version;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    lint::LintManager,
    position::Position,
    reporting::{Code, Codes, Processed, Severity},
};

/// Parse a config file
///
/// # Errors
/// If the file is invalid
///
/// # Panics
/// If the localizations mutex is poisoned
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
            let default_enabled = project.is_some_and(|p| p.runtime().is_pedantic());
            let mut manager = LintManager::new(
                project.map_or_else(Default::default, |project| project.lints().config().clone()),
            );
            manager.extend(
                analyze::CONFIG_LINTS
                    .iter()
                    .map(|l| (**l).clone())
                    .collect::<Vec<_>>(),
                default_enabled,
            )?;
            let localizations = Arc::new(Mutex::new(vec![]));
            let codes = config.analyze(
                &LintData {
                    path: String::new(),
                    localizations: localizations.clone(),
                },
                project,
                processed,
                &manager,
            );
            Ok(ConfigReport {
                codes,
                patches: config.get_patches(),
                localized: Arc::<Mutex<Vec<(String, Position)>>>::try_unwrap(localizations)
                    .expect("not poisoned")
                    .into_inner()
                    .expect("not poisoned"),
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
    localized: Vec<(String, Position)>,
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
    /// Get the hints and notes
    pub fn notes_and_helps(&self) -> Vec<&Arc<dyn Code>> {
        self.codes
            .iter()
            .filter(|c| c.severity() == Severity::Help || c.severity() == Severity::Note)
            .collect::<Vec<_>>()
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

    #[must_use]
    /// Get the localized strings
    pub fn localized(&self) -> &[(String, Position)] {
        &self.localized
    }
}
