pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

use std::sync::{Arc, Mutex};

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::Addon,
    lint::LintManager,
    lint_manager,
    position::Position,
    reporting::{Codes, Processed},
};
use lints::s02_event_handlers::{
    EventHandlerRunner, LintS02EventIncorrectCommand, LintS02EventInsufficientVersion,
    LintS02EventUnknown,
};

use crate::{
    BinaryCommand, Expression, NularCommand, Statement, Statements, UnaryCommand,
    parser::database::Database,
};

lint_manager!(
    sqf,
    vec![(
        vec![
            Arc::new(Box::new(LintS02EventUnknown)),
            Arc::new(Box::new(LintS02EventIncorrectCommand)),
            Arc::new(Box::new(LintS02EventInsufficientVersion)),
        ],
        Box::new(EventHandlerRunner),
    )]
);

#[must_use]
/// Analyze a set of statements
///
/// # Panics
/// If the localizations mutex is poisoned
pub fn analyze(
    statements: &Statements,
    project: Option<&ProjectConfig>,
    processed: &Processed,
    addon: Arc<Addon>,
    database: Arc<Database>,
) -> (Codes, Localizations) {
    let default_enabled = project.is_some_and(|p| p.runtime().is_pedantic());
    let mut manager: LintManager<LintData> = LintManager::new(
        project.map_or_else(Default::default, |project| project.lints().sqf().clone()),
    );
    if let Err(lint_errors) = manager.extend(
        SQF_LINTS.iter().map(|l| (**l).clone()).collect::<Vec<_>>(),
        default_enabled,
    ) {
        return (lint_errors, vec![]);
    }
    if let Err(lint_errors) = manager.push_group(
        vec![
            Arc::new(Box::new(LintS02EventUnknown)),
            Arc::new(Box::new(LintS02EventIncorrectCommand)),
            Arc::new(Box::new(LintS02EventInsufficientVersion)),
        ],
        Box::new(EventHandlerRunner),
        default_enabled,
    ) {
        return (lint_errors, vec![]);
    }
    let localizations = Arc::new(Mutex::new(vec![]));
    let codes = statements.analyze(
        &LintData {
            addon,
            database,
            localizations: localizations.clone(),
        },
        project,
        processed,
        &manager,
    );
    (
        codes,
        Arc::<Mutex<Vec<(String, Position)>>>::try_unwrap(localizations)
            .expect("not poisoned")
            .into_inner()
            .expect("not poisoned"),
    )
}

pub type Localizations = Vec<(String, Position)>;
pub struct LintData {
    pub(crate) addon: Arc<Addon>,
    pub(crate) database: Arc<Database>,
    pub(crate) localizations: Arc<Mutex<Localizations>>,
}

pub trait Analyze: Sized + 'static {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        codes
    }
}

impl Analyze for NularCommand {}
impl Analyze for UnaryCommand {}
impl Analyze for BinaryCommand {}

impl Analyze for Statements {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        for statement in self.content() {
            codes.extend(statement.analyze(data, project, processed, manager));
        }
        codes
    }
}

impl Analyze for Statement {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        match self {
            Self::Expression(exp, _)
            | Self::AssignLocal(_, exp, _)
            | Self::AssignGlobal(_, exp, _) => {
                codes.extend(exp.analyze(data, project, processed, manager));
            }
        }
        codes
    }
}

impl Analyze for Expression {
    fn analyze(
        &self,
        data: &LintData,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<LintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(data, project, Some(processed), self));
        match self {
            Self::Array(exp, _) => {
                for e in exp {
                    codes.extend(e.analyze(data, project, processed, manager));
                }
            }
            Self::Code(s) => codes.extend(s.analyze(data, project, processed, manager)),
            Self::NularCommand(nc, _) => {
                codes.extend(nc.analyze(data, project, processed, manager));
            }
            Self::UnaryCommand(uc, exp, _) => {
                codes.extend(uc.analyze(data, project, processed, manager));
                codes.extend(exp.analyze(data, project, processed, manager));
            }
            Self::BinaryCommand(bc, exp_left, exp_right, _) => {
                codes.extend(bc.analyze(data, project, processed, manager));
                codes.extend(exp_left.analyze(data, project, processed, manager));
                codes.extend(exp_right.analyze(data, project, processed, manager));
            }
            _ => {}
        }
        codes
    }
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
