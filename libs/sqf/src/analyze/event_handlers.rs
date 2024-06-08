use std::{ops::Range, sync::Arc};

use arma3_wiki::model::{EventHandlerNamespace, ParsedEventHandler};
use hemtt_workspace::{
    addons::Addon,
    reporting::{Code, Processed},
};

use crate::{
    analyze::{
        codes::{saw1_unknown_event::UnknownEvent, saw2_wrong_event_command::WrongEventCommand},
        extract_constant,
    },
    parser::database::Database,
    BinaryCommand, Expression, Statements, UnaryCommand,
};

use super::{
    codes::sae2_require_version_event::InsufficientRequiredVersionEvent, WarningAndErrors,
};

pub fn event_handlers(
    addon: Option<&Addon>,
    statements: &Statements,
    processed: &Processed,
    database: &Database,
) -> WarningAndErrors {
    let mut warnings: Vec<Arc<dyn Code>> = Vec::new();
    let mut errors: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            let Some((ns, name, id, target)) = get_namespaces(expression) else {
                continue;
            };
            if ns.is_empty() {
                continue;
            }
            if name.contains("UserAction") {
                // Requires arma3-wiki to parse and provide https://community.bistudio.com/wiki/inputAction/actions
                continue;
            }
            let eh = database.wiki().event_handler(&id.0);
            warnings.extend(check_unknown(
                &ns,
                &name,
                &id,
                target.map(|t| &**t),
                &eh,
                processed,
                database,
            ));
            errors.extend(check_version(
                addon, &ns, &name, &id, &eh, processed, database,
            ));
        }
    }
    (warnings, errors)
}

fn check_unknown(
    ns: &[EventHandlerNamespace],
    name: &str,
    id: &(Arc<str>, &Range<usize>),
    target: Option<&Expression>,
    eh: &[(EventHandlerNamespace, &ParsedEventHandler)],
    processed: &Processed,
    database: &Database,
) -> Vec<Arc<dyn Code>> {
    if eh.is_empty() {
        return vec![Arc::new(UnknownEvent::new(
            ns,
            id.1.clone(),
            name.to_owned(),
            id.0.clone(),
            processed,
            database,
        ))];
    }

    if ns.iter().any(|n| eh.iter().any(|(ns, _)| ns == n)) {
        return Vec::new();
    }
    vec![Arc::new(WrongEventCommand::new(
        id.1.clone(),
        name.to_owned(),
        id.0.clone(),
        target.and_then(extract_constant),
        eh.iter().map(|(ns, _)| ns).copied().collect(),
        processed,
        database,
    ))]
}

fn check_version(
    addon: Option<&Addon>,
    ns: &[EventHandlerNamespace],
    name: &str,
    id: &(Arc<str>, &Range<usize>),
    eh: &[(EventHandlerNamespace, &ParsedEventHandler)],
    processed: &Processed,
    database: &Database,
) -> Vec<Arc<dyn Code>> {
    let Some(addon) = addon else {
        return Vec::new();
    };
    let Some(required) = addon.build_data().required_version() else {
        // TODO what to do here?
        return Vec::new();
    };
    let Some((_, eh)) = eh.iter().find(|(ins, _)| ns.contains(ins)) else {
        return Vec::new();
    };
    let Some(since) = eh.since() else {
        return Vec::new();
    };
    let Some(version) = since.arma_3() else {
        return Vec::new();
    };
    let mut errors: Vec<Arc<dyn Code>> = Vec::new();
    let wiki_version = arma3_wiki::model::Version::new(
        u8::try_from(required.0.major()).unwrap_or_default(),
        u8::try_from(required.0.minor()).unwrap_or_default(),
    );
    let required = (wiki_version, required.1, required.2);
    if wiki_version < *version {
        errors.push(Arc::new(InsufficientRequiredVersionEvent::new(
            name.to_owned(),
            id.1.clone(),
            *version,
            required,
            *database.wiki().version(),
            processed,
        )));
    }
    errors
}

#[allow(clippy::type_complexity)]
fn get_namespaces(
    expression: &Expression,
) -> Option<(
    Vec<EventHandlerNamespace>,
    String,
    (Arc<str>, &Range<usize>),
    Option<&Box<Expression>>,
)> {
    match expression {
        Expression::BinaryCommand(BinaryCommand::Named(name), target, id, _) => Some((
            EventHandlerNamespace::by_command(name),
            name.to_owned(),
            get_id(id)?,
            Some(target),
        )),
        Expression::UnaryCommand(UnaryCommand::Named(name), id, _) => Some((
            EventHandlerNamespace::by_command(name),
            name.to_owned(),
            get_id(id)?,
            None,
        )),
        _ => None,
    }
}

fn get_id(expression: &Expression) -> Option<(Arc<str>, &Range<usize>)> {
    match expression {
        Expression::String(id, span, _) => Some((id.clone(), span)),
        Expression::Array(items, _) => {
            if items.is_empty() {
                None
            } else {
                get_id(&items[0])
            }
        }
        _ => None,
    }
}
