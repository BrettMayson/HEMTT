#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub mod analyze;
mod model;
pub mod parse;
pub mod rapify;
pub use model::*;

use analyze::{Analyze, CfgPatch, ChumskyCode, LintData};
use chumsky::Parser;
use hemtt_common::version::Version;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::{Addon, DefinedFunctions, MagazineWellInfo},
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
            let functions_defined = Arc::new(Mutex::new(HashSet::new()));
            let magazine_well_info = Arc::new(Mutex::new((Vec::new(), Vec::new())));
            let codes = config.analyze(
                &LintData {
                    path: String::new(),
                    localizations: localizations.clone(),
                    functions_defined: functions_defined.clone(),
                    magazine_well_info: magazine_well_info.clone(),
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
                functions_defined: Arc::<Mutex<DefinedFunctions>>::try_unwrap(functions_defined)
                    .expect("not poisoned")
                    .into_inner()
                    .expect("not poisoned"),
                magazine_well_info: Arc::<Mutex<MagazineWellInfo>>::try_unwrap(magazine_well_info)
                    .expect("not poisoned")
                    .into_inner()
                    .expect("not poisoned"),
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
    /// # Panics
    pub fn push_to_addon(&self, addon: &Addon) {
        let build_data = addon.build_data();
        build_data
            .localizations()
            .lock()
            .expect("not poisoned")
            .extend(
                self.localized
                    .iter()
                    .map(|(s, p)| (s.to_owned(), p.clone())),
            );
        build_data
            .functions_defined()
            .lock()
            .expect("not poisoned")
            .extend(self.functions_defined.clone());
        let (magazines, magwell_codes) = self.magazine_well_info.clone();
        build_data
            .magazine_well_info()
            .lock()
            .expect("not poisoned")
            .0
            .extend(magazines);
        build_data
            .magazine_well_info()
            .lock()
            .expect("not poisoned")
            .1
            .extend(magwell_codes);
    }
}
