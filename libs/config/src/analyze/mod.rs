use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::{Addon, DefinedFunctions, MagazineWellInfo},
    lint::LintManager,
    lint_manager,
    position::Position,
    reporting::{Codes, Processed},
};

mod cfgpatch;
mod chumsky;

pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

pub struct LintData {
    pub(crate) path: String,
    pub(crate) localizations: Arc<Mutex<Vec<(String, Position)>>>,
    pub(crate) functions_defined: Arc<Mutex<DefinedFunctions>>,
    pub(crate) magazine_well_info: Arc<Mutex<MagazineWellInfo>>,
}

lint_manager!(config, vec![]);

pub use cfgpatch::CfgPatch;
pub use chumsky::ChumskyCode;

use crate::{Array, Class, Config, Expression, Item, Number, Property, Str, Value};

/// Trait for rapifying objects
pub trait Analyze: Sized + 'static {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes
    }
}

impl Analyze for Str {}
impl Analyze for Number {}
impl Analyze for Expression {}

impl Analyze for Config {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(manager.run(data, project, Some(processed), &self.to_class()));
        codes.extend(
            self.0
                .iter()
                .flat_map(|p| p.analyze(data, project, processed, manager)),
        );
        codes
    }
}

impl Analyze for Class {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } | Self::Root { properties, .. } => {
                let data = LintData {
                    path: self.name().map_or_else(
                        || data.path.clone(),
                        |name| format!("{}/{}", data.path, name.value),
                    ),
                    localizations: data.localizations.clone(),
                    functions_defined: data.functions_defined.clone(),
                    magazine_well_info: data.magazine_well_info.clone(),
                };
                properties
                    .iter()
                    .flat_map(|p| p.analyze(&data, project, processed, manager))
                    .collect::<Vec<_>>()
            }
        });
        codes
    }
}

impl Analyze for Property {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(match self {
            Self::Entry { value, .. } => {
                let data = LintData {
                    path: format!("{}.{}", data.path, self.name().value),
                    localizations: data.localizations.clone(),
                    functions_defined: data.functions_defined.clone(),
                    magazine_well_info: data.magazine_well_info.clone(),
                };
                value.analyze(&data, project, processed, manager)
            }
            Self::Class(c) => c.analyze(data, project, processed, manager),
            Self::Delete(_) | Self::MissingSemicolon(_, _) => vec![],
        });
        codes
    }
}

impl Analyze for Value {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(match self {
            Self::Str(s) => s.analyze(data, project, processed, manager),
            Self::Number(n) => n.analyze(data, project, processed, manager),
            Self::Expression(e) => e.analyze(data, project, processed, manager),
            Self::Array(a) | Self::UnexpectedArray(a) => {
                a.analyze(data, project, processed, manager)
            }
            Self::Invalid(_) => {
                vec![]
            }
        });
        codes
    }
}

impl Analyze for Array {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(
            self.items
                .iter()
                .flat_map(|i| i.analyze(data, project, processed, manager)),
        );
        codes
    }
}

impl Analyze for Item {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(match self {
            Self::Str(s) => s.analyze(data, project, processed, manager),
            Self::Number(n) => n.analyze(data, project, processed, manager),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.analyze(data, project, processed, manager))
                .collect::<Vec<_>>(),
            Self::Invalid(_) => {
                vec![]
            }
        });
        codes
    }
}

#[must_use]
#[allow(clippy::ptr_arg)]
pub fn lint_all(project: Option<&ProjectConfig>, addons: &Vec<Addon>) -> Codes {
    let default_enabled = project.is_some_and(|p| p.runtime().is_pedantic());
    let mut manager = LintManager::new(
        project.map_or_else(Default::default, |project| project.lints().config().clone()),
    );
    let _e = manager.extend(
        crate::analyze::CONFIG_LINTS
            .iter()
            .map(|l| (**l).clone())
            .collect::<Vec<_>>(),
        default_enabled,
    );

    manager.run(
        &LintData {
            path: String::new(),
            localizations: Arc::new(Mutex::new(vec![])),
            functions_defined: Arc::new(Mutex::new(HashSet::new())),
            magazine_well_info: Arc::new(Mutex::new((Vec::new(), Vec::new()))),
        },
        project,
        None,
        addons,
    )
}
