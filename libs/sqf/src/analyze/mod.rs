mod statements;

pub mod codes;

use std::sync::Arc;

use hemtt_common::{
    addons::Addon,
    project::ProjectConfig,
    reporting::{Code, Processed},
};

use crate::{parser::database::Database, Statements};

type Codes = Vec<Arc<dyn Code>>;

pub trait Analyze {
    /// Check if the object is valid and can be rapified
    fn valid(&self, project: Option<&ProjectConfig>) -> bool;

    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        addon: &Addon,
        database: &Database,
    ) -> Codes;

    fn errors(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        addon: &Addon,
        database: &Database,
    ) -> Codes;
}

#[must_use]
pub fn analyze(
    statements: &Statements,
    project: Option<&ProjectConfig>,
    processed: &Processed,
    addon: &Addon,
    database: &Database,
) -> (Codes, Codes) {
    (
        statements.warnings(project, processed, addon, database),
        statements.errors(project, processed, addon, database),
    )
}
