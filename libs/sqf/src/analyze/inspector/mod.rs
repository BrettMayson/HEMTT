//! Inspects code, checking code args and variable usage
//!
use std::{hash::Hash, ops::Range, sync::OnceLock, vec};

use crate::{
    BinaryCommand, Expression, Statement, Statements, UnaryCommand,
    analyze::inspector::game_value::NilSource, parser::database::Database,
};
use game_value::GameValue;
use hemtt_workspace::reporting::Processed;
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
#[allow(unused_imports)]
use tracing::trace;
use tracing::warn;

mod commands;
mod external_functions;
mod game_value;
mod issue;
pub use issue::{InvalidArgs, Issue};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VarSource {
    Assignment(Range<usize>, Range<usize>),
    ForLoop(Range<usize>),
    Params(Range<usize>),
    Private(Range<usize>),
    Magic(Range<usize>),
    Ignore,
}
impl VarSource {
    #[must_use]
    pub const fn skip_errors(&self) -> bool {
        matches!(self, Self::Magic(..)) || matches!(self, Self::Ignore)
    }
    #[must_use]
    pub fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Assignment(range, _)
            | Self::ForLoop(range)
            | Self::Params(range)
            | Self::Private(range)
            | Self::Magic(range) => Some(range.clone()),
            Self::Ignore => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarHolder {
    possible: IndexSet<GameValue>,
    usage: i32,
    source: VarSource,
}

pub type Stack = IndexMap<String, VarHolder>;

/// Scope where variables and possible returns types are tracked
pub struct ScriptScope {
    vars_local: Vec<Stack>,
    /// Set of possible return values from this scope
    returns_set: Vec<IndexSet<GameValue>>,
    /// Error suppression for scopes below this one (for trial runs of loops)
    errors_suppressed: Vec<bool>,
    /// Orphan scopes are code blocks that are created but don't appear to be called in a known way
    is_orphan_scope: bool,
}

impl ScriptScope {
    /// # Panics
    pub fn add_returns(&mut self, values: IndexSet<GameValue>) {
        self.returns_set
            .last_mut()
            .expect("stack not empty")
            .extend(values);
    }
}

pub struct Inspector<'a> {
    errors: IndexSet<Issue>,
    vars_global: Stack,
    ignored_vars: IndexSet<String>,
    code_seen: IndexSet<Expression>,
    code_used: IndexSet<Expression>,
    code_active: IndexSet<Expression>,
    scopes: Vec<ScriptScope>,
    database: &'a Database,
}

impl<'a> Inspector<'a> {
    #[must_use]
    pub fn new(ignored_vars: &IndexSet<String>, database: &'a Database) -> Self {
        let mut inspector = Self {
            errors: IndexSet::new(),
            vars_global: Stack::new(),
            ignored_vars: ignored_vars.clone(),
            code_seen: IndexSet::new(),
            code_used: IndexSet::new(),
            code_active: IndexSet::new(),
            scopes: Vec::new(),
            database,
        };
        inspector.scope_push(false);
        inspector
    }
    /// # Panics
    #[must_use]
    pub fn active_scope(&mut self) -> &mut ScriptScope {
        self.scopes.last_mut().expect("there is always a scope")
    }
    pub fn scope_push(&mut self, is_orphan_scope: bool) {
        // println!("Creating ScriptScope, orphan: {is_orphan_scope}");
        let scope = ScriptScope {
            vars_local: Vec::new(),
            returns_set: Vec::new(),
            is_orphan_scope,
            errors_suppressed: Vec::new(),
        };
        self.scopes.push(scope);
        self.stack_push(None, false);
        for var in &self.ignored_vars.clone() {
            self.var_assign(
                var,
                true,
                IndexSet::from([GameValue::Anything]),
                VarSource::Ignore,
            );
        }
    }
    pub fn scope_pop(&mut self) {
        let _ = self.stack_pop(None);
        debug_assert!(self.scopes.last().is_some_and(|s| s.vars_local.is_empty()));
        self.scopes.pop();
    }
    #[must_use]
    pub fn finish(mut self) -> Vec<Issue> {
        self.scope_pop();
        debug_assert!(self.scopes.is_empty());
        let unused = &self.code_seen - &self.code_used;
        for code in &unused {
            let Expression::Code(statements) = code else {
                unreachable!("only code");
            };
            self.code_used(code);
            // println!("-- Checking external scope");
            self.scope_push(true); // create orphan scope
            self.eval_statements(statements, false);
            self.scope_pop();
        }
        self.errors.into_iter().collect()
    }

    pub fn code_seen(&mut self, expression: &Expression) {
        self.code_seen.insert(expression.clone());
    }
    pub fn code_used(&mut self, expression: &Expression) {
        debug_assert!(self.code_seen.contains(expression));
        self.code_used.insert(expression.clone());
    }
    pub fn error_insert(&mut self, issue: Issue) {
        // skip if errors are suppressed in any parent scope
        if self
            .active_scope()
            .errors_suppressed
            .iter()
            .rev()
            .skip(1)
            .all(|s| !*s)
        {
            self.errors.insert(issue);
        }
    }

    pub fn stack_push(
        &mut self,
        expression_opt: Option<&Expression>,
        suppress_errors: bool,
    ) -> Option<usize> {
        // println!("-- Stack Push {}", self.active_scope().vars_local.len());
        let return_index = match expression_opt {
            Some(expression) => {
                self.code_used(expression);
                let (index, inserted) = self.code_active.insert_full(expression.clone());
                if !inserted {
                    return None; // already active, prevent infinite recursion
                }
                Some(index)
            }
            None => None,
        };
        self.active_scope().vars_local.push(Stack::new());
        self.active_scope().returns_set.push(IndexSet::new());
        self.active_scope().errors_suppressed.push(suppress_errors);
        return_index
    }
    /// # Panics
    pub fn stack_pop(&mut self, index: Option<usize>) -> IndexSet<GameValue> {
        if let Some(index) = index {
            let _ = self.code_active.swap_remove_index(index);
        }
        // Check for unused vars in this stack level
        for (var, holder) in self
            .active_scope()
            .vars_local
            .pop()
            .expect("there is always a stack")
        {
            if holder.usage == 0
                && !holder.source.skip_errors()
                && !self.ignored_vars.contains(&var)
            {
                self.error_insert(Issue::Unused(var, holder.source, false));
            }
        }
        self.active_scope().errors_suppressed.pop();
        let mut returns_set = self
            .active_scope()
            .returns_set
            .pop()
            .expect("stack to exist");
        if returns_set.is_empty() {
            returns_set.insert(GameValue::Nothing(NilSource::EmptyStack));
        }
        returns_set
    }

    pub fn var_assign(
        &mut self,
        var: &str,
        local: bool,
        possible_values: IndexSet<GameValue>,
        source: VarSource,
    ) {
        // println!("var_assign: `{var}` local:{local} lvl: {}", self.local.len());
        let var_lower = var.to_ascii_lowercase();
        if !var_lower.starts_with('_') {
            let holder = self
                .vars_global
                .entry(var_lower)
                .or_insert_with(|| VarHolder {
                    possible: IndexSet::new(),
                    usage: 0,
                    source,
                });
            holder.possible.extend(possible_values);
            return;
        }
        // read-only borrow to compute stack position with a minimal lifetime so we can
        // insert errors (which needs &mut self) before taking a mutable borrow again.
        let stack_level_search: Option<usize>;
        let mut stack_level;
        {
            let current_vars = &self.active_scope().vars_local;
            stack_level_search = current_vars
                .iter()
                .rev()
                .position(|s| s.contains_key(&var_lower));
            stack_level = current_vars.len().saturating_sub(1);
        }
        if stack_level_search.is_none() {
            if !local {
                self.error_insert(Issue::NotPrivate(
                    var.to_owned(),
                    source.get_range().unwrap_or_default(),
                ));
            }
        } else if !local {
            stack_level -= stack_level_search.unwrap_or_default();
        }
        // If we are writing to a variable at our current level
        let error_opt: Option<Issue> = if stack_level_search == Some(0)
            && !source.skip_errors()
            && let Some(current_vars) = self.active_scope().vars_local.last()
            && let Some(existing_var) = current_vars.get(&var_lower)
            && !existing_var.source.skip_errors()
        {
            if existing_var.usage == 0 && !matches!(existing_var.source, VarSource::Private(_)) {
                Some(Issue::Unused(
                    var.to_owned(),
                    existing_var.source.clone(),
                    true, // overwritten before use
                ))
            } else if local {
                // shadowed (only at same level and not also unused)
                Some(Issue::Shadowed(
                    var.to_owned(),
                    source.get_range().unwrap_or_default(),
                ))
            } else {
                None
            }
        } else {
            None
        };
        if let Some(error) = error_opt {
            self.error_insert(error);
        }
        let vars_local = &mut self.active_scope().vars_local;
        let holder = vars_local[stack_level]
            .entry(var_lower)
            .or_insert_with(|| VarHolder {
                possible: IndexSet::new(),
                usage: 0,
                source: source.clone(),
            });
        if stack_level_search.unwrap_or_default() == 0 {
            // Brand new or at same level as origin, totally replace possible values
            holder.possible = possible_values;
        } else {
            // In a inner scope, just extend possible values
            holder.possible.extend(possible_values);
        }
    }

    #[must_use]
    /// # Panics
    pub fn var_retrieve(
        &mut self,
        var: &str,
        source: &Range<usize>,
        peek: bool,
    ) -> IndexSet<(GameValue, Range<usize>)> {
        let var_lower = var.to_ascii_lowercase();
        let holder_option = if var_lower.starts_with('_') {
            let stack_level_search = self
                .active_scope()
                .vars_local
                .iter()
                .rev()
                .position(|s| s.contains_key(&var_lower));
            let mut stack_level = self.active_scope().vars_local.len() - 1;
            if let Some(stack_level_search) = stack_level_search {
                stack_level -= stack_level_search;
            } else if !peek {
                let is_oprhan = self.active_scope().is_orphan_scope;
                self.errors
                    .insert(Issue::Undefined(var.to_owned(), source.clone(), is_oprhan));
            }
            self.active_scope().vars_local[stack_level].get_mut(&var_lower)
        } else if self.vars_global.contains_key(&var_lower) {
            self.vars_global.get_mut(&var_lower)
        } else {
            return IndexSet::from([(GameValue::Anything, source.clone())]);
        };
        if let Some(holder) = holder_option {
            holder.usage += 1;
            let mut set = holder.possible.clone();

            if !var_lower.starts_with('_') && self.ignored_vars.contains(&var_lower) {
                // Assume that a ignored global var could be anything
                set.insert(GameValue::Anything);
            }
            // println!("var_retrieve: `{var}` {set:?}");
            set.into_iter().map(|gv| (gv, source.clone())).collect()
        } else {
            // we've reported the error above, just return Any so it doesn't fail everything after
            IndexSet::from([(GameValue::Anything, source.clone())])
        }
    }

    /// Checks for bad argument values if the syntax was otherwise valid (e.g. cmd that takes anything)
    fn eval_check_bad_args(
        &mut self,
        debug_type: &str,
        source: &Range<usize>,
        expression: &Expression,
        result_set: &IndexSet<GameValue>,
    ) {
        // there should never be any valid reason to have these as inputs
        if result_set.iter().any(GameValue::is_poison_nil) {
            self.error_insert(Issue::InvalidArgs {
                command: debug_type.to_string(),
                span: source.clone(),
                variant: InvalidArgs::NilResultUsed {
                    found: result_set.iter().cloned().collect(),
                    span: expression.span(),
                },
            });
        }
    }

    #[must_use]
    #[allow(unused_assignments)]
    /// Evaluate expression in current scope
    pub fn eval_expression(
        &mut self,
        expression: &Expression,
    ) -> IndexSet<(GameValue, Range<usize>)> {
        fn command_return(
            cmd_set: IndexSet<GameValue>,
            return_set: Option<IndexSet<GameValue>>,
            source: &Range<usize>,
        ) -> IndexSet<(GameValue, Range<usize>)> {
            // Use custom return if it exists or just use wiki set
            let mut set = return_set.unwrap_or(cmd_set);
            // If a command could return multiple values, make the nil results generic
            if set.len() > 1 && set.swap_remove(&GameValue::Nothing(NilSource::CommandReturn)) {
                set.insert(GameValue::Nothing(NilSource::Generic));
            }
            set.into_iter().map(|gv| (gv, source.clone())).collect()
        }
        let mut debug_type = String::new();
        let possible_values = match expression {
            Expression::Variable(var, source) => self.var_retrieve(var, source, false),
            Expression::Number(_, source) => {
                IndexSet::from([(GameValue::Number(Some(expression.clone())), source.clone())])
            }
            Expression::Boolean(_, source) => {
                IndexSet::from([(GameValue::Boolean(Some(expression.clone())), source.clone())])
            }
            Expression::String(_, source, _) => {
                IndexSet::from([(GameValue::String(Some(expression.clone())), source.clone())])
            }
            Expression::Array(array, source) => {
                let gv_array: Vec<Vec<(GameValue, Range<usize>)>> = array
                    .iter()
                    .map(|e| self.eval_expression(e).into_iter().collect())
                    .collect();
                IndexSet::from([(GameValue::Array(Some(gv_array), None), source.clone())])
            }
            Expression::NularCommand(cmd, source) => {
                debug_type = format!("[N:{}]", cmd.as_str());
                let mut cmd_set = GameValue::from_cmd(expression, None, None, self.database).0;
                if cmd_set.is_empty() {
                    warn!("cmd_set is empty on nular command `{}`", cmd.as_str()); // should not be possible
                    cmd_set.insert(GameValue::Anything); // don't cause confusing errors for code downstream
                }
                let return_set = if cmd.as_str().eq_ignore_ascii_case("nil") {
                    // nil returns a explicit Nothing
                    Some(IndexSet::from([GameValue::Nothing(NilSource::ExplicitNil)]))
                } else {
                    None
                };
                command_return(cmd_set, return_set, source)
            }
            Expression::UnaryCommand(cmd, rhs, source) => {
                debug_type = format!("[U:{}]", cmd.as_str());
                let rhs_set = self
                    .eval_expression(rhs)
                    .into_iter()
                    .map(|(gv, _)| gv)
                    .collect();
                let (mut cmd_set, _, expected_rhs) =
                    GameValue::from_cmd(expression, None, Some(&rhs_set), self.database);
                if cmd_set.is_empty() {
                    self.error_insert(Issue::InvalidArgs {
                        command: debug_type.clone(),
                        span: source.clone(),
                        variant: InvalidArgs::TypeNotExpected {
                            expected: expected_rhs.into_iter().collect(),
                            found: rhs_set.iter().cloned().collect(),
                            span: source.clone(),
                        },
                    });
                    cmd_set.insert(GameValue::Anything); // don't cause confusing errors for code downstream
                } else {
                    self.eval_check_bad_args(&debug_type, source, rhs, &rhs_set);
                }
                let return_set = match cmd {
                    UnaryCommand::Named(named) => match named.to_ascii_lowercase().as_str() {
                        "params" => Some(self.cmd_generic_params(&rhs_set, &debug_type, source)),
                        "private" => Some(self.cmd_u_private(&rhs_set)),
                        "call" => Some(self.cmd_generic_call(&rhs_set, None, false)),
                        "default" => {
                            let returns = self.cmd_generic_call(&rhs_set, None, false);
                            self.active_scope().add_returns(returns);
                            None
                        }
                        "isnil" => Some(self.cmd_u_is_nil(&rhs_set)),
                        "while" | "waituntil" => {
                            let _ = self.cmd_generic_call(&rhs_set, None, true); // loop
                            None
                        }
                        "try" => {
                            let _ = self.cmd_generic_call(&rhs_set, None, false);
                            None
                        }
                        "for" => Some(self.cmd_for(&rhs_set)),
                        "tostring" => Some(self.cmd_u_to_string(&rhs_set)),
                        "addmissioneventhandler" => {
                            for possible in rhs_set {
                                let GameValue::Array(Some(gv_array), _) = possible else {
                                    continue;
                                };
                                if gv_array.len() > 1 {
                                    self.external_new_scope(
                                        &gv_array[1],
                                        &vec![
                                            ("_this", GameValue::Anything),
                                            ("_thisEvent", GameValue::String(None)),
                                            ("_thisEventHandler", GameValue::Number(None)),
                                            ("_thisArgs", GameValue::Anything), // gv_array[2]?
                                        ],
                                    );
                                }
                            }
                            None
                        }
                        _ => None,
                    },
                    _ => None,
                };
                command_return(cmd_set, return_set, source)
            }
            Expression::BinaryCommand(cmd, lhs, rhs, source) => {
                debug_type = format!("[B:{}]", cmd.as_str());
                let lhs_set = self
                    .eval_expression(lhs)
                    .into_iter()
                    .map(|(gv, _)| gv)
                    .collect();
                let rhs_set = self
                    .eval_expression(rhs)
                    .into_iter()
                    .map(|(gv, _)| gv)
                    .collect();
                let (mut cmd_set, expected_lhs, expected_rhs) =
                    GameValue::from_cmd(expression, Some(&lhs_set), Some(&rhs_set), self.database);
                if cmd_set.is_empty() {
                    // we must have invalid args
                    if !expected_lhs.is_empty() {
                        self.error_insert(Issue::InvalidArgs {
                            command: debug_type.clone(),
                            span: source.clone(),
                            variant: InvalidArgs::TypeNotExpected {
                                expected: expected_lhs.into_iter().collect(),
                                found: lhs_set.iter().cloned().collect(),
                                span: lhs.span(),
                            },
                        });
                    }
                    if !expected_rhs.is_empty() {
                        self.error_insert(Issue::InvalidArgs {
                            command: debug_type.clone(),
                            span: source.clone(),
                            variant: InvalidArgs::TypeNotExpected {
                                expected: expected_rhs.into_iter().collect(),
                                found: rhs_set.iter().cloned().collect(),
                                span: rhs.span(),
                            },
                        });
                    }
                    cmd_set.insert(GameValue::Anything); // don't cause confusing errors for code downstream
                } else {
                    self.eval_check_bad_args(&debug_type, source, lhs, &lhs_set);
                    self.eval_check_bad_args(&debug_type, source, rhs, &rhs_set);
                }
                let return_set = match cmd {
                    BinaryCommand::Associate => {
                        // the : from case ToDo: these run outside of the do scope
                        let returns = self.cmd_generic_call(&rhs_set, None, false);
                        self.active_scope().add_returns(returns);
                        None
                    }
                    BinaryCommand::And | BinaryCommand::Or => {
                        let _ = self.cmd_generic_call(&rhs_set, None, false);
                        None
                    }
                    BinaryCommand::Else => Some(self.cmd_b_else(&lhs_set, &rhs_set, source)),
                    BinaryCommand::Eq => {
                        self.cmd_eqx_count_lint(lhs, rhs, true);
                        None
                    }
                    BinaryCommand::Greater | BinaryCommand::NotEq => {
                        self.cmd_eqx_count_lint(lhs, rhs, false);
                        None
                    }
                    BinaryCommand::Named(named) => match named.to_ascii_lowercase().as_str() {
                        "set" | "pushback" | "pushbackunique" | "append" | "resize" => {
                            // these commands modify the LHS by lvalue, assume it is now a generic array
                            self.cmd_generic_modify_lvalue(lhs);
                            None
                        }
                        "params" => Some(self.cmd_generic_params(&rhs_set, &debug_type, source)),
                        "call" => {
                            self.external_function(&lhs_set, rhs);
                            Some(self.cmd_generic_call(&rhs_set, None, false))
                        }
                        "spawn" | "addpublicvariableeventhandler" => {
                            self.external_new_scope(
                                &rhs_set.into_iter().map(|gv| (gv, source.clone())).collect(),
                                &vec![],
                            );
                            None
                        }
                        "exitwith" => {
                            let returns = self.cmd_generic_call(&rhs_set, None, false);
                            self.active_scope().add_returns(returns);
                            None
                        }
                        "do" => {
                            // from While, With, For, and Switch
                            // todo: handle switch return value
                            Some(self.cmd_b_do(&lhs_set, &rhs_set, true))
                        }
                        "from" | "to" | "step" => Some(self.cmd_b_from_chain(&lhs_set, &rhs_set)),
                        "then" => Some(self.cmd_b_then(rhs, &rhs_set)),
                        "foreach" | "foreachreversed" => {
                            let mut magic = vec![
                                ("_x", GameValue::get_array_value_type(&rhs_set)),
                                ("_forEachIndex", GameValue::Number(None)),
                            ];
                            if !rhs_set.iter().all(|gv| matches!(gv, GameValue::Array(..))) {
                                magic.push(("_y", GameValue::Anything));
                            }
                            Some(self.cmd_generic_call(&lhs_set, Some((&magic, source)), true))
                        }
                        "count" => {
                            let magic = vec![("_x", GameValue::get_array_value_type(&rhs_set))];
                            let _ = self.cmd_generic_call(&lhs_set, Some((&magic, source)), true);
                            None
                        }
                        "apply" => {
                            let mut magic = vec![("_x", GameValue::get_array_value_type(&lhs_set))];
                            if !lhs_set.iter().all(|gv| matches!(gv, GameValue::Array(..))) {
                                magic.push(("_y", GameValue::Anything));
                            }
                            let _ = self.cmd_generic_call(&rhs_set, Some((&magic, source)), true);
                            None
                        }
                        "findif" => {
                            let magic = vec![("_x", GameValue::get_array_value_type(&lhs_set))];
                            let _ = self.cmd_generic_call(&rhs_set, Some((&magic, source)), true);
                            None
                        }
                        "try" => {
                            let _ = self.cmd_generic_call(&rhs_set, None, false);
                            None
                        }
                        "catch" => {
                            let magic = vec![("_exception", GameValue::Anything)];
                            let _ = self.cmd_generic_call(&rhs_set, Some((&magic, source)), false);
                            None
                        }
                        "getordefaultcall" => Some(self.cmd_b_get_or_default_call(&rhs_set)),
                        "select" => Some(self.cmd_b_select(&lhs_set, &rhs_set, &cmd_set, source)),
                        "addeventhandler"
                        | "addmpeventhandler"
                        | "ctrladdeventhandler"
                        | "displayaddeventhandler" => {
                            for possible in rhs_set {
                                let GameValue::Array(Some(gv_array), _) = possible else {
                                    continue;
                                };
                                if gv_array.len() > 1 {
                                    self.external_new_scope(
                                        &gv_array[1],
                                        &vec![
                                            ("_this", GameValue::Anything),
                                            ("_thisEvent", GameValue::String(None)),
                                            ("_thisEventHandler", GameValue::Number(None)),
                                        ],
                                    );
                                }
                            }
                            None
                        }
                        _ => None,
                    },
                    _ => None,
                };
                command_return(cmd_set, return_set, source)
            }
            Expression::Code(statements) => {
                self.code_seen(expression);
                debug_type = format!("CODE:{}", statements.content().len());
                IndexSet::from([(GameValue::Code(Some(expression.clone())), statements.span())])
            }
            Expression::ConsumeableArray(_, _) => unreachable!(""),
        };
        #[cfg(debug_assertions)]
        trace!(
            "eval expression{}->{:?}",
            debug_type,
            possible_values
                .iter()
                .map(|(gv, _)| format!("{gv:?}"))
                .collect::<Vec<_>>()
        );
        #[allow(clippy::let_and_return)]
        possible_values
    }

    /// Evaluate statements in the current scope and return possible return values of last command
    fn eval_statements(&mut self, statements: &Statements, add_to_stack: bool) {
        let mut last_statement = IndexSet::new();
        for statement in statements.content() {
            last_statement = match statement {
                Statement::AssignGlobal(var, expression, source) => {
                    // x or _x
                    let possible_values = self
                        .eval_expression(expression)
                        .into_iter()
                        .map(|(gv, _)| gv)
                        .collect();
                    self.eval_check_bad_args("=", source, expression, &possible_values);
                    self.var_assign(
                        var,
                        false,
                        possible_values,
                        VarSource::Assignment(
                            source.start..source.start + var.len(),
                            expression.span().clone(),
                        ),
                    );
                    IndexSet::from([GameValue::Assignment])
                }
                Statement::AssignLocal(var, expression, source) => {
                    // private _x
                    let possible_values = self
                        .eval_expression(expression)
                        .into_iter()
                        .map(|(gv, _)| gv)
                        .collect();
                    self.eval_check_bad_args("=", source, expression, &possible_values);
                    self.var_assign(
                        var,
                        true,
                        possible_values,
                        VarSource::Assignment(
                            8 + source.start..8 + source.start + var.len(),
                            expression.span().clone(),
                        ),
                    );
                    IndexSet::from([GameValue::Assignment])
                }
                Statement::Expression(expression, _) => self
                    .eval_expression(expression)
                    .into_iter()
                    .map(|(gv, _)| gv)
                    .collect(),
            }
        }
        if add_to_stack {
            self.active_scope().add_returns(last_statement);
        }
    }
}

#[must_use]
/// Run statements and return issues
/// # Panics
pub fn run_processed(
    statements: &Statements,
    processed: &Processed,
    database: &Database,
) -> Vec<Issue> {
    static RE_IGNORE_VARIABLES: OnceLock<Regex> = OnceLock::new();
    static RE_IGNORE_VARIABLE_ENTRIES: OnceLock<Regex> = OnceLock::new();

    let mut ignored_vars = IndexSet::new();
    ignored_vars.insert("_this".to_ascii_lowercase());
    ignored_vars.insert("_fnc_scriptName".to_ascii_lowercase()); // may be set via cfgFunctions
    ignored_vars.insert("_fnc_scriptNameParent".to_ascii_lowercase());
    let re1 = RE_IGNORE_VARIABLES.get_or_init(|| {
        Regex::new(r"(?:\#pragma hemtt ignore_variables|\/\/ ?IGNORE_PRIVATE_WARNING) ?\[(.*)\]")
            .expect("regex ok")
    });
    let re2 =
        RE_IGNORE_VARIABLE_ENTRIES.get_or_init(|| Regex::new(r#""(.*?)""#).expect("regex ok"));
    for (_path, raw_source) in processed.sources() {
        for (_, [ignores]) in re1.captures_iter(&raw_source).map(|c| c.extract()) {
            for (_, [var]) in re2.captures_iter(ignores).map(|c| c.extract()) {
                ignored_vars.insert(var.to_ascii_lowercase());
            }
        }
    }

    let mut inspector = Inspector::new(&ignored_vars, database);
    inspector.eval_statements(statements, true);
    let issues = inspector.finish();
    // for ig in ignored_vars.clone() {
    //     if ig == "_this" || ig == "_fnc_scriptname" || ig == "_fnc_scriptnameparent" {
    //         continue;
    //     }
    //     let mut igtest = ignored_vars.clone();
    //     igtest.shift_remove(&ig);
    //     let mut inspector = Inspector::new(&igtest);
    //     inspector.eval_statements(statements, true, database);
    //     let test_issues = inspector.finish(database);
    //     let path = processed.sources();
    //     if test_issues.len() < issues.len() {
    //         println!(
    //             "in {:?}-{:?} and {} is undeeded [{}->{}]",
    //             path[0].0,
    //             path[path.len() - 1].0,
    //             ig,
    //             issues.len(),
    //             test_issues.len()
    //         );
    //     }
    // }
    #[allow(clippy::let_and_return)]
    issues
}
