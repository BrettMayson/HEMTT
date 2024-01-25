pub mod codes;

mod find_in_str;
mod if_assign;
mod required_version;
mod select_parse_number;
mod str_format;
mod typename;

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
        addon: Option<&Addon>,
        database: &Database,
    ) -> Codes;

    fn errors(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        addon: Option<&Addon>,
        database: &Database,
    ) -> Codes;
}

#[must_use]
pub fn analyze(
    statements: &Statements,
    _project: Option<&ProjectConfig>,
    processed: &Processed,
    addon: Option<&Addon>,
    database: &Database,
) -> (Codes, Codes) {
    (
        {
            let mut warnings = Vec::new();
            warnings.extend(if_assign::if_assign(statements, processed));
            warnings.extend(find_in_str::find_in_str(statements, processed));
            warnings.extend(typename::typename(statements, processed));
            // warnings.extend(str_format::str_format(statements, processed)); // Too many false positives for now
            warnings.extend(select_parse_number::select_parse_number(
                statements, processed,
            ));
            warnings
        },
        {
            let mut errors = Vec::new();
            errors.extend(required_version::required_version(
                statements, processed, addon, database,
            ));
            errors
        },
    )
}
