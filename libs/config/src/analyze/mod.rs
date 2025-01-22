use hemtt_common::config::{BuildInfo, ProjectConfig};
use hemtt_workspace::{
    lint::LintManager,
    lint_manager,
    reporting::{Codes, Processed},
};

mod cfgpatch;
mod chumsky;

pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

pub struct LintData {
    pub(crate) path: String,
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
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, build_info, Some(processed), self));
        codes
    }
}

impl Analyze for Str {}
impl Analyze for Number {}
impl Analyze for Expression {}

impl Analyze for Config {
    fn analyze(
        &self,
        _data: &LintData,
        project: Option<&ProjectConfig>,
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let data = LintData {
            path: String::new(),
        };
        let mut codes = vec![];
        codes.extend(manager.run(&data, project, build_info, Some(processed), self));
        codes.extend(manager.run(
            &data,
            project,
            build_info,
            Some(processed),
            &self.to_class(),
        ));
        codes.extend(
            self.0
                .iter()
                .flat_map(|p| p.analyze(&data, project, build_info, processed, manager)),
        );
        codes
    }
}

impl Analyze for Class {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, build_info, Some(processed), self));
        codes.extend(match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } | Self::Root { properties, .. } => {
                let data = LintData {
                    path: self.name().map_or_else(
                        || data.path.clone(),
                        |name| format!("{}/{}", data.path, name.value),
                    ),
                };
                properties
                    .iter()
                    .flat_map(|p| p.analyze(&data, project, build_info, processed, manager))
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
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, build_info, Some(processed), self));
        codes.extend(match self {
            Self::Entry { value, .. } => {
                let data = LintData {
                    path: format!("{}.{}", data.path, self.name().value),
                };
                value.analyze(&data, project, build_info, processed, manager)
            }
            Self::Class(c) => c.analyze(data, project, build_info, processed, manager),
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
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, build_info, Some(processed), self));
        codes.extend(match self {
            Self::Str(s) => s.analyze(data, project, build_info, processed, manager),
            Self::Number(n) => n.analyze(data, project, build_info, processed, manager),
            Self::Expression(e) => e.analyze(data, project, build_info, processed, manager),
            Self::Array(a) | Self::UnexpectedArray(a) => {
                a.analyze(data, project, build_info, processed, manager)
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
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, build_info, Some(processed), self));
        codes.extend(
            self.items
                .iter()
                .flat_map(|i| i.analyze(data, project, build_info, processed, manager)),
        );
        codes
    }
}

impl Analyze for Item {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        build_info: Option<&BuildInfo>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, build_info, Some(processed), self));
        codes.extend(match self {
            Self::Str(s) => s.analyze(data, project, build_info, processed, manager),
            Self::Number(n) => n.analyze(data, project, build_info, processed, manager),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.analyze(data, project, build_info, processed, manager))
                .collect::<Vec<_>>(),
            Self::Invalid(_) => {
                vec![]
            }
        });
        codes
    }
}
