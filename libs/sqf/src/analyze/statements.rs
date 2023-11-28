use std::ops::Range;

use hemtt_common::{
    addons::Addon,
    project::ProjectConfig,
    reporting::{Code, Processed},
    version::Version,
};

use crate::{parser::database::Database, Statements};

use super::Analyze;

impl Analyze for Statements {
    fn valid(&self, project: Option<&ProjectConfig>) -> bool {
        true
    }

    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        addon: &Addon,
        database: &Database,
    ) -> Vec<Box<dyn Code>> {
        vec![]
    }

    fn errors(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        addon: &Addon,
        database: &Database,
    ) -> Vec<Box<dyn Code>> {
        let mut errors = Vec::new();
        errors.extend(required_version(
            addon.build_data().required_version().unwrap_or_default(),
            self,
            database,
        ));
        errors
    }
}

fn required_version(
    required: (Version, String, Range<usize>),
    statements: &Statements,
    database: &Database,
) -> Vec<Box<dyn Code>> {
    let mut errors: Vec<Box<dyn Code>> = Vec::new();
    let wiki_version = a3_wiki::model::Version::new(
        u8::try_from(required.0.major()).unwrap_or_default(),
        u8::try_from(required.0.minor()).unwrap_or_default(),
    );
    let required = (wiki_version, required.1, required.2);
    let (command, usage, usage_span) = statements.required_version(database);
    if wiki_version < usage {
        errors.push(Box::new(
            super::codes::sae1_require_version::InsufficientRequiredVersion::new(
                command, usage_span, usage, required, *database.wiki().version(),
            ),
        ));
    }
    errors
}
