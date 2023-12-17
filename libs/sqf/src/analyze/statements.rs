use std::{ops::Range, sync::Arc};

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
    ) -> Vec<Arc<dyn Code>> {
        vec![]
    }

    fn errors(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        addon: &Addon,
        database: &Database,
    ) -> Vec<Arc<dyn Code>> {
        let mut errors = Vec::new();
        errors.extend(required_version(
            addon.build_data().required_version().unwrap_or_default(),
            self,
            processed,
            database,
        ));
        errors
    }
}

fn required_version(
    required: (Version, String, Range<usize>),
    statements: &Statements,
    processed: &Processed,
    database: &Database,
) -> Vec<Arc<dyn Code>> {
    let mut errors: Vec<Arc<dyn Code>> = Vec::new();
    let wiki_version = arma3_wiki::model::Version::new(
        u8::try_from(required.0.major()).unwrap_or_default(),
        u8::try_from(required.0.minor()).unwrap_or_default(),
    );
    let required = (wiki_version, required.1, required.2);
    let (command, usage, usage_span) = statements.required_version(database);
    if wiki_version < usage {
        errors.push(Arc::new(
            super::codes::sae1_require_version::InsufficientRequiredVersion::new(
                command,
                usage_span,
                usage,
                required,
                *database.wiki().version(),
                processed,
            ),
        ));
    }
    errors
}
