pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

pub mod inspector;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use hemtt_common::config::{ProjectConfig, RuntimeArguments};
use hemtt_workspace::{
    addons::{Addon, DefinedFunctions, UsedFunctions},
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
) -> (Codes, Option<SqfReport>) {
    let mut manager: LintManager<LintData> = LintManager::new(
        project.map_or_else(Default::default, |project| project.lints().sqf().clone()),
        project.map_or_else(RuntimeArguments::default, |p| p.runtime().clone()),
    );
    if let Err(lint_errors) =
        manager.extend(SQF_LINTS.iter().map(|l| (**l).clone()).collect::<Vec<_>>())
    {
        return (lint_errors, None);
    }
    if let Err(lint_errors) = manager.push_group(
        vec![
            Arc::new(Box::new(LintS02EventUnknown)),
            Arc::new(Box::new(LintS02EventIncorrectCommand)),
            Arc::new(Box::new(LintS02EventInsufficientVersion)),
        ],
        Box::new(EventHandlerRunner),
    ) {
        return (lint_errors, None);
    }
    let localizations = Arc::new(Mutex::new(vec![]));
    let functions_used = Arc::new(Mutex::new(vec![]));
    let functions_defined = Arc::new(Mutex::new(HashSet::new()));
    let codes = statements.analyze(
        &LintData {
            addon: Some(addon),
            database,
            localizations: localizations.clone(),
            functions_used: functions_used.clone(),
            functions_defined: functions_defined.clone(),
        },
        project,
        processed,
        &manager,
    );

    let localizations = Arc::<Mutex<Localizations>>::try_unwrap(localizations)
        .expect("not poisoned")
        .into_inner()
        .expect("not poisoned");
    let functions_used = Arc::<Mutex<UsedFunctions>>::try_unwrap(functions_used)
        .expect("not poisoned")
        .into_inner()
        .expect("not poisoned");
    let functions_defined = Arc::<Mutex<DefinedFunctions>>::try_unwrap(functions_defined)
        .expect("not poisoned")
        .into_inner()
        .expect("not poisoned");
    (
        codes,
        Some(SqfReport {
            localizations,
            functions_used,
            functions_defined,
        }),
    )
}

pub type Localizations = Vec<(String, Position)>;
pub struct LintData {
    pub(crate) addon: Option<Arc<Addon>>,
    pub(crate) database: Arc<Database>,
    pub(crate) localizations: Arc<Mutex<Localizations>>,
    pub(crate) functions_used: Arc<Mutex<UsedFunctions>>,
    pub(crate) functions_defined: Arc<Mutex<DefinedFunctions>>,
}
pub struct SqfReport {
    localizations: Localizations,
    functions_used: UsedFunctions,
    functions_defined: DefinedFunctions,
}

impl SqfReport {
    /// Pushes the report into an Addon
    /// # Panics
    pub fn push_to_addon(&self, addon: &Addon) {
        let build_data = addon.build_data();
        build_data
            .localizations()
            .lock()
            .expect("not poisoned")
            .extend(self.localizations.clone());
        build_data
            .functions_used()
            .lock()
            .expect("not poisoned")
            .extend(self.functions_used.clone());
        addon
            .build_data()
            .functions_defined()
            .lock()
            .expect("not poisoned")
            .extend(self.functions_defined.clone());
    }

    #[must_use]
    pub const fn localizations(&self) -> &Localizations {
        &self.localizations
    }

    #[must_use]
    pub const fn functions_used(&self) -> &UsedFunctions {
        &self.functions_used
    }

    #[must_use]
    pub const fn functions_defined(&self) -> &DefinedFunctions {
        &self.functions_defined
    }
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
#[must_use]
fn extract_constant(expression: &Expression) -> Option<(String, bool)> {
    if let Expression::Code(code) = &expression
        && code.content.len() == 1
        && let Statement::Expression(expr, _) = &code.content[0]
    {
        return match expr {
            Expression::Boolean(bool, _) => Some((bool.to_string(), false)),
            Expression::Number(num, _) => Some((num.0.to_string(), false)),
            Expression::String(string, _, _) => Some((string.to_string(), true)),
            Expression::Variable(var, _) => Some((var.clone(), false)),
            _ => None,
        };
    }
    None
}
#[must_use]
fn check_expression_deep(expression: &Expression, f: &impl Fn(&Expression) -> bool) -> bool {
    match expression {
        Expression::Array(elements, _) => {
            for element in elements {
                if check_expression_deep(element, f) { return true }
            }
        }
        Expression::Code(statements) => {
            for statement in &statements.content {
                match statement {
                    Statement::Expression(expr, _) | Statement::AssignLocal(_, expr, _) | Statement::AssignGlobal(_, expr, _) => {
                        if check_expression_deep(expr, f) { return true }
                    }
                }
            }
        }
        Expression::UnaryCommand(_, expr, _) => {
            if check_expression_deep(expr, f) { return true }
        }
        Expression::BinaryCommand(_, left, right, _) => {
            if check_expression_deep(left, f) { return true }
            if check_expression_deep(right, f) { return true }
        }
        _ => {}
    }
    f(expression)
}

#[must_use]
#[allow(clippy::ptr_arg)]
pub fn lint_all(
    project_config: Option<&ProjectConfig>,
    addons: &Vec<Addon>,
    database: Arc<Database>,
) -> Codes {
    let mut manager = LintManager::new(
        project_config.map_or_else(Default::default, |project| project.lints().sqf().clone()),
        project_config.map_or_else(RuntimeArguments::default, |p| p.runtime().clone()),
    );
    if let Err(e) = manager.extend(SQF_LINTS.iter().map(|l| (**l).clone()).collect::<Vec<_>>()) {
        return e;
    }

    manager.run(
        &LintData {
            addon: None,
            database,
            localizations: Arc::new(Mutex::new(vec![])),
            functions_used: Arc::new(Mutex::new(vec![])),
            functions_defined: Arc::new(Mutex::new(HashSet::new())),
        },
        project_config,
        None,
        addons,
    )
}
