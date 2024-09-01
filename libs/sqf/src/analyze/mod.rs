pub mod codes;

mod command_case;
mod event_handlers;
mod find_in_str;
mod if_assign;
mod required_version;
mod select_parse_number;
mod str_format;
mod typename;

use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::Addon,
    reporting::{Code, Processed},
};

use crate::{parser::database::Database, Expression, Statement, Statements};

type Codes = Vec<Arc<dyn Code>>;
type WarningAndErrors = (Codes, Codes);

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
    let (mut warnings, mut errors) =
        event_handlers::event_handlers(addon, statements, processed, database);
    (
        {
            warnings.extend(if_assign::if_assign(statements, processed));
            warnings.extend(find_in_str::find_in_str(statements, processed));
            warnings.extend(typename::typename(statements, processed));
            // warnings.extend(str_format::str_format(statements, processed)); // Too many false positives for now
            warnings.extend(select_parse_number::select_parse_number(
                statements, processed, database,
            ));
            warnings.extend(command_case::command_case(statements, processed, database));
            warnings
        },
        {
            errors.extend(required_version::required_version(
                statements, processed, addon, database,
            ));
            errors
        },
    )
}

/// Extracts a constant from an expression
///
/// Returns a tuple of the constant and a boolean indicating if quotes are needed
fn extract_constant(expression: &Expression) -> Option<(String, bool)> {
    if let Expression::Code(code) = &expression {
        if code.content.len() == 1 {
            if let Statement::Expression(expr, _) = &code.content[0] {
                return match expr {
                    Expression::Boolean(bool, _) => Some((bool.to_string(), false)),
                    Expression::Number(num, _) => Some((num.0.to_string(), false)),
                    Expression::String(string, _, _) => Some((string.to_string(), true)),
                    Expression::Variable(var, _) => Some((var.to_string(), false)),
                    _ => None,
                };
            }
        }
    }
    None
}
