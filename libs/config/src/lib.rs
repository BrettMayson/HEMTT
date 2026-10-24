#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

use std::sync::Arc;

pub mod analyze;
pub mod check;
pub mod display;
pub mod files;
mod model;
pub mod parse;
pub mod rapify;

pub use model::*;

use analyze::{Analyze, CfgPatch, ChumskyCode, LintData};
use chumsky::Parser;
use hemtt_common::{config::RuntimeArguments, version::Version};

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::{Addon, DefinedFunctions, Localizations, MagazineWellInfo},
    lint::LintManager,
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
            let mut manager = LintManager::new(
                project.map_or_else(Default::default, |project| project.lints().config().clone()),
                project.map_or_else(RuntimeArguments::default, |p| p.runtime().clone()),
            );
            manager.extend(
                analyze::CONFIG_LINTS
                    .iter()
                    .map(|l| (**l).clone())
                    .collect::<Vec<_>>(),
            )?;
            let lint_data = LintData::default();
            let codes = config.analyze(&lint_data, project, processed, &manager);
            let (localized, functions_defined, magazine_well_info) = lint_data.unpack();
            Ok(ConfigReport {
                codes,
                patches: config.get_patches(),
                localized,
                config,
                functions_defined,
                magazine_well_info,
            })
        },
    )
}

/// A parsed config file with warnings and errors
pub struct ConfigReport {
    config: Config,
    codes: Codes,
    patches: Vec<CfgPatch>,
    localized: Localizations,
    functions_defined: DefinedFunctions,
    magazine_well_info: MagazineWellInfo,
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

    /// Pushes the report's data into an Addon
    pub fn push_to_addon(self, addon: &Addon) {
        let build_data = addon.build_data();
        if !self.localized.is_empty()
            && let Ok(mut lock) = build_data.localizations().lock()
        {
            lock.extend(self.localized);
        }
        if !self.functions_defined.is_empty()
            && let Ok(mut lock) = build_data.functions_defined().lock()
        {
            lock.extend(self.functions_defined);
        }
        let (magazines, magwell_codes) = self.magazine_well_info;
        if !(magazines.is_empty() && magwell_codes.is_empty())
            && let Ok(mut lock) = build_data.magazine_well_info().lock()
        {
            lock.0.extend(magazines);
            lock.1.extend(magwell_codes);
        }
    }

    #[must_use]
    /// Get the `DefinedFunctions`
    pub const fn functions_defined(&self) -> &DefinedFunctions {
        &self.functions_defined
    }

    #[must_use]
    /// Get the `Localizations`
    pub const fn localizations(&self) -> &Localizations {
        &self.localized
    }

    #[must_use]
    /// Get the `MagazineWellInfo`
    pub const fn magazine_well_info(&self) -> &MagazineWellInfo {
        &self.magazine_well_info
    }
}
