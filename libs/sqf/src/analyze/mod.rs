pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

pub mod inspector;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use hemtt_common::{
    config::{ProjectConfig, RuntimeArguments},
    toml_lint::{TomlLintDef, TomlLintSqfTarget},
};
use hemtt_workspace::{
    addons::{Addon, DefinedFunctions, Localizations, UsedFunctions},
    lint::LintManager,
    lint_manager,
    reporting::{Code, Codes, Processed, TomlLintCode},
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
    manager: Option<Arc<LintManager<LintData>>>,
) -> (Codes, Option<SqfReport>) {
    let manager: Arc<LintManager<LintData>> = if let Some(manager) = manager {
        manager
    } else {
        match create_lint_manager(project) {
            Ok(manager) => Arc::new(manager),
            Err(codes) => return (codes, None),
        }
    };
    let lint_data = LintData {
        addon: Some(addon),
        database,
        localizations: Arc::default(),
        functions_used: Arc::default(),
        functions_defined: Arc::default(),
    };
    let codes = statements.analyze(&lint_data, project, processed, &manager);
    let (localizations, functions_used, functions_defined) = lint_data.unpack();
    (
        codes,
        Some(SqfReport {
            localizations,
            functions_used,
            functions_defined,
        }),
    )
}

pub fn analyze_toml(
    _statements: &Statements,
    source: &str,
    path: &hemtt_workspace::WorkspacePath,
    project: Option<&ProjectConfig>,
    _processed: &Processed,
) -> Codes {
    let Some(lints) = project.map(hemtt_common::config::ProjectConfig::toml_lints) else {
        return Vec::new();
    };
    if lints.is_empty() {
        return Vec::new();
    }
    let mut codes = Codes::new();
    for lint in lints {
        let Some(TomlLintDef::Patterns(patterns)) = lint.sqf().get(&TomlLintSqfTarget::File) else {
            continue;
        };
        for range in lint.run_file(source, patterns) {
            codes.push(Arc::new(TomlLintCode::new_file(lint, range, path)));
        }
    }
    codes
}

#[must_use]
/// Try to recover the original source text for a span
///
/// If the span came from a macro expansion, this looks up the original source
/// text before expansion. This is useful for showing macro names in suggestions.
pub fn recover_original_source(processed: &Processed, span_start: usize) -> Option<String> {
    // Try to find a mapping at the start of the span
    if let Some(mapping) = processed.mapping(span_start)
        && mapping.was_macro()
    {
        // This came from a macro, get the original source
        let source_info = processed.source(mapping.source())?;
        let source_text = &source_info.1;

        // Extract the original text using the original position
        let original_start = mapping.original_start();
        let original_end = mapping.original_end();

        // Get the original token's symbol as a fallback
        let token = mapping.token();
        let macro_text = token.symbol();

        // Try to find the full macro call in the original source
        // Look forward from the original position to find the closing paren/bracket
        if original_start < source_text.len() {
            // Start with the macro name
            let mut end = original_end;

            // Look ahead for parentheses (macro arguments)
            while end < source_text.len()
                && source_text
                    .chars()
                    .nth(end)
                    .is_some_and(|c| c == ' ' || c == '\t')
            {
                end += 1;
            }

            // If we find an opening paren, include everything until the closing paren
            if end < source_text.len() && source_text.chars().nth(end) == Some('(') {
                let mut paren_count = 0;
                for (i, ch) in source_text[end..].chars().enumerate() {
                    if ch == '(' {
                        paren_count += 1;
                    } else if ch == ')' {
                        paren_count -= 1;
                        if paren_count == 0 {
                            end = end + i + 1;
                            break;
                        }
                    }
                }
            }

            // Ensure we have valid indices
            if original_start <= source_text.len()
                && end <= source_text.len()
                && original_start <= end
            {
                let original_text = source_text
                    .chars()
                    .skip(original_start)
                    .take(end - original_start)
                    .collect::<String>();
                return Some(original_text.trim().to_string());
            }
        }

        // Fallback: just use the token's symbol
        return Some(macro_text.to_string().trim().to_string());
    }
    None
}

pub struct LintData {
    pub(crate) addon: Option<Arc<Addon>>,
    pub(crate) database: Arc<Database>,
    pub(crate) localizations: Arc<Mutex<Localizations>>,
    pub(crate) functions_used: Arc<Mutex<UsedFunctions>>,
    pub(crate) functions_defined: Arc<Mutex<DefinedFunctions>>,
}
impl LintData {
    fn unpack(self) -> (Localizations, UsedFunctions, DefinedFunctions) {
        debug_assert_eq!(Arc::strong_count(&self.localizations), 1);
        (
            Arc::try_unwrap(self.localizations)
                .expect("not poisoned")
                .into_inner()
                .expect("not poisoned"),
            Arc::try_unwrap(self.functions_used)
                .expect("not poisoned")
                .into_inner()
                .expect("not poisoned"),
            Arc::try_unwrap(self.functions_defined)
                .expect("not poisoned")
                .into_inner()
                .expect("not poisoned"),
        )
    }
}
pub struct SqfReport {
    localizations: Localizations,
    functions_used: UsedFunctions,
    functions_defined: DefinedFunctions,
}

impl SqfReport {
    /// Pushes the report into an Addon
    pub fn push_to_addon(self, addon: &Addon) {
        let build_data = addon.build_data();
        if !self.localizations.is_empty()
            && let Ok(mut lock) = build_data.localizations().lock()
        {
            lock.extend(self.localizations);
        }
        if !self.functions_used.is_empty()
            && let Ok(mut lock) = build_data.functions_used().lock()
        {
            lock.extend(self.functions_used);
        }
        if !self.functions_defined.is_empty()
            && let Ok(mut lock) = build_data.functions_defined().lock()
        {
            lock.extend(self.functions_defined);
        }
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

#[must_use]
/// Extracts a constant from an expression
///
/// Returns a tuple of the constant and a boolean indicating if quotes are needed
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
/// Deeply collects all results from the expression tree that match the given pattern function.
fn pattern_collect<T>(expr: &Expression, f: &impl Fn(&Expression) -> Option<T>) -> Vec<T> {
    fn collect_inner<T>(
        expr: &Expression,
        f: &impl Fn(&Expression) -> Option<T>,
        results: &mut Vec<T>,
    ) {
        if let Some(result) = f(expr) {
            results.push(result);
        }
        match expr {
            Expression::UnaryCommand(_, rhs, _) => {
                collect_inner(rhs, f, results);
            }
            Expression::BinaryCommand(_, lhs, rhs, _) => {
                collect_inner(lhs, f, results);
                collect_inner(rhs, f, results);
            }
            Expression::Code(statements) => {
                for stmt in &statements.content {
                    match stmt {
                        Statement::Expression(inner, _)
                        | Statement::AssignLocal(_, inner, _)
                        | Statement::AssignGlobal(_, inner, _) => {
                            collect_inner(inner, f, results);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let mut results = Vec::new();
    collect_inner(expr, f, &mut results);
    results
}

#[must_use]
#[allow(clippy::ptr_arg)]
pub fn lint_all(
    project_config: Option<&ProjectConfig>,
    addons: &Vec<Addon>,
    database: Arc<Database>,
    manager: &LintManager<LintData>,
) -> Codes {
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

/// Creates and configures the SQF lint manager.
/// # Errors
/// Returns Lint Manager or Err containing the lint error codes.
pub fn create_lint_manager(
    project: Option<&ProjectConfig>,
) -> Result<LintManager<LintData>, Vec<Arc<dyn Code>>> {
    let mut manager: LintManager<LintData> = LintManager::new(
        project.map_or_else(Default::default, |project| project.lints().sqf().clone()),
        project.map_or_else(RuntimeArguments::default, |p| p.runtime().clone()),
    );
    manager.extend(SQF_LINTS.iter().map(|l| (**l).clone()).collect::<Vec<_>>())?;
    manager.push_group(
        vec![
            Arc::new(Box::new(LintS02EventUnknown)),
            Arc::new(Box::new(LintS02EventIncorrectCommand)),
            Arc::new(Box::new(LintS02EventInsufficientVersion)),
        ],
        Box::new(EventHandlerRunner),
    )?;
    let codes = manager.check_config_usage("sqf", "s");
    if !codes.is_empty() {
        return Err(codes);
    }
    Ok(manager)
}
