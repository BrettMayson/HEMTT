use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    lint::LintManager,
    reporting::{Code, Processed},
};

mod cfgpatch;
mod chumsky;
pub mod lints;

pub use cfgpatch::CfgPatch;
pub use chumsky::ChumskyCode;

use crate::{Array, Class, Config, Expression, Item, Number, Property, Str, Value};

/// Trait for rapifying objects
pub trait Analyze: Sized + 'static {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        codes
    }
}

impl Analyze for Str {}
impl Analyze for Number {}
impl Analyze for Expression {}

impl Analyze for Config {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        let root_class = self.to_class();
        codes.extend(manager.run(project, &root_class));
        codes.extend(manager.run_processed(project, processed, &root_class));
        codes.extend(
            self.0
                .iter()
                .flat_map(|p| p.analyze(project, processed, manager)),
        );
        codes
    }
}

impl Analyze for Class {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        codes.extend(match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } | Self::Root { properties, .. } => properties
                .iter()
                .flat_map(|p| p.analyze(project, processed, manager))
                .collect::<Vec<_>>(),
        });
        codes
    }
}

impl Analyze for Property {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        codes.extend(match self {
            Self::Entry { value, .. } => value.analyze(project, processed, manager),
            Self::Class(c) => c.analyze(project, processed, manager),
            Self::Delete(_) | Self::MissingSemicolon(_, _) => vec![],
        });
        codes
    }
}

impl Analyze for Value {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        codes.extend(match self {
            Self::Str(s) => s.analyze(project, processed, manager),
            Self::Number(n) => n.analyze(project, processed, manager),
            Self::Expression(e) => e.analyze(project, processed, manager),
            Self::Array(a) | Self::UnexpectedArray(a) => a.analyze(project, processed, manager),
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
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        codes.extend(
            self.items
                .iter()
                .flat_map(|i| i.analyze(project, processed, manager)),
        );
        codes
    }
}

impl Analyze for Item {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager,
    ) -> Vec<Arc<dyn Code>> {
        let mut codes = vec![];
        codes.extend(manager.run(project, self));
        codes.extend(manager.run_processed(project, processed, self));
        codes.extend(match self {
            Self::Str(s) => s.analyze(project, processed, manager),
            Self::Number(n) => n.analyze(project, processed, manager),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.analyze(project, processed, manager))
                .collect::<Vec<_>>(),
            Self::Invalid(_) => {
                vec![]
            }
        });
        codes
    }
}
