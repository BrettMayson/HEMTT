pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::Addon,
    lint::LintManager,
    reporting::{Codes, Processed},
};
use lints::s02_event_handlers::{
    EventHandlerRunner, LintS02EventIncorrectCommand, LintS02EventInsufficientVersion,
    LintS02EventUnknown,
};

use crate::{
    parser::database::Database, BinaryCommand, Expression, NularCommand, Statement, Statements,
    UnaryCommand,
};

#[linkme::distributed_slice]
pub static SQF_LINTS: [std::sync::LazyLock<
    std::sync::Arc<Box<dyn hemtt_workspace::lint::Lint<SqfLintData>>>,
>];

#[macro_export]
macro_rules! lint {
    ($name:ident) => {
        #[allow(clippy::module_name_repetitions)]
        pub struct $name;
        #[linkme::distributed_slice($crate::analyze::SQF_LINTS)]
        static LINT_ADD: std::sync::LazyLock<
            std::sync::Arc<Box<dyn hemtt_workspace::lint::Lint<$crate::analyze::SqfLintData>>>,
        > = std::sync::LazyLock::new(|| std::sync::Arc::new(Box::new($name)));
    };
}

#[must_use]
pub fn analyze(
    statements: &Statements,
    project: Option<&ProjectConfig>,
    processed: &Processed,
    addon: Arc<Addon>,
    database: Arc<Database>,
) -> Codes {
    let mut manager: LintManager<SqfLintData> = LintManager::new(
        project.map_or_else(Default::default, |project| project.lints().config().clone()),
        (addon, database),
    );
    if let Err(lint_errors) =
        manager.extend(SQF_LINTS.iter().map(|l| (**l).clone()).collect::<Vec<_>>())
    {
        return lint_errors;
    }
    if let Err(lint_errors) = manager.push_group(
        vec![
            Arc::new(Box::new(LintS02EventUnknown)),
            Arc::new(Box::new(LintS02EventIncorrectCommand)),
            Arc::new(Box::new(LintS02EventInsufficientVersion)),
        ],
        Box::new(EventHandlerRunner),
    ) {
        return lint_errors;
    }
    statements.analyze(project, processed, &manager)
}

pub type SqfLintData = (Arc<Addon>, Arc<Database>);

pub trait Analyze: Sized + 'static {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(project, Some(processed), self));
        codes
    }
}

impl Analyze for NularCommand {}
impl Analyze for UnaryCommand {}
impl Analyze for BinaryCommand {}

impl Analyze for Statements {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(project, Some(processed), self));
        for statement in self.content() {
            codes.extend(statement.analyze(project, processed, manager));
        }
        codes
    }
}

impl Analyze for Statement {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(project, Some(processed), self));
        match self {
            Self::Expression(exp, _)
            | Self::AssignLocal(_, exp, _)
            | Self::AssignGlobal(_, exp, _) => {
                codes.extend(exp.analyze(project, processed, manager));
            }
        }
        codes
    }
}

impl Analyze for Expression {
    fn analyze(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
        manager: &LintManager<SqfLintData>,
    ) -> Codes {
        let mut codes = vec![];
        codes.extend(manager.run(project, Some(processed), self));
        match self {
            Self::Array(exp, _) => {
                for e in exp {
                    codes.extend(e.analyze(project, processed, manager));
                }
            }
            Self::Code(s) => codes.extend(s.analyze(project, processed, manager)),
            Self::NularCommand(nc, _) => {
                codes.extend(nc.analyze(project, processed, manager));
            }
            Self::UnaryCommand(uc, exp, _) => {
                codes.extend(uc.analyze(project, processed, manager));
                codes.extend(exp.analyze(project, processed, manager));
            }
            Self::BinaryCommand(bc, exp_left, exp_right, _) => {
                codes.extend(bc.analyze(project, processed, manager));
                codes.extend(exp_left.analyze(project, processed, manager));
                codes.extend(exp_right.analyze(project, processed, manager));
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
