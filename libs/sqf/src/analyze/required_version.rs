use std::sync::Arc;

use hemtt_common::{
    addons::Addon,
    reporting::{Code, Processed},
};

use crate::{parser::database::Database, Statements};

pub fn required_version(
    statements: &Statements,
    processed: &Processed,
    addon: Option<&Addon>,
    database: &Database,
) -> Vec<Arc<dyn Code>> {
    let Some(addon) = addon else {
        return Vec::new();
    };
    let required = addon.build_data().required_version().unwrap_or_default();
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
