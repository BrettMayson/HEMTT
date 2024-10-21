use hemtt_common::config::ProjectConfig;
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

pub struct SqfLintData {}

lint_manager!(config, vec![]);

pub use cfgpatch::CfgPatch;
pub use chumsky::ChumskyCode;

use crate::{Array, Class, Config, Expression, Item, Number, Property, Str, Value};

/// Trait for rapifying objects
pub trait Analyze: Sized + 'static {
    fn analyze(
        &self,
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
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
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
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
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } | Self::Root { properties, .. } => properties
                .iter()
                .flat_map(|p| p.analyze(data, project, processed, manager))
                .collect::<Vec<_>>(),
        });
        codes
    }
}

impl Analyze for Property {
    fn analyze(
        &self,
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes.extend(match self {
            Self::Entry { value, .. } => value.analyze(data, project, processed, manager),
            Self::Class(c) => c.analyze(data, project, processed, manager),
            Self::Delete(_) | Self::MissingSemicolon(_, _) => vec![],
        });
        codes
    }
}

impl Analyze for Value {
    fn analyze(
        &self,
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
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
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
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
        data: &SqfLintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
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
