//! Inspects code, checking code args and variable usage
//!
use std::{cell::RefCell, hash::Hash, ops::Range, rc::Rc, vec};

use crate::{
    BinaryCommand, Expression, Statement, Statements, UnaryCommand, parser::database::Database,
};
use game_value::GameValue;
use hemtt_workspace::reporting::Processed;
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use tracing::trace;

mod commands;
mod external_functions;
mod game_value;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Issue {
    InvalidArgs(String, Range<usize>),
    Undefined(String, Range<usize>, bool),
    Unused(String, VarSource),
    Shadowed(String, Range<usize>),
    NotPrivate(String, Range<usize>),
    CountArrayComparison(bool, Range<usize>, String),
}

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

pub struct SciptScope {
    errors: IndexSet<Issue>,
    global: Rc<RefCell<Stack>>,
    local: Vec<Stack>,
    code_seen: IndexSet<Expression>,
    code_used: IndexSet<Expression>,
    /// Orphan scopes are code blocks that are created but don't appear to be called in a known way
    is_orphan_scope: bool,
    ignored_vars: IndexSet<String>,
}

impl SciptScope {
    #[must_use]
    pub fn create(
        global: Rc<RefCell<Stack>>,
        ignored_vars: &IndexSet<String>,
        is_orphan_scope: bool,
    ) -> Self {
        // trace!("Creating ScriptScope");
        let mut scope = Self {
            errors: IndexSet::new(),
            global,
            local: Vec::new(),
            code_seen: IndexSet::new(),
            code_used: IndexSet::new(),
            is_orphan_scope,
            ignored_vars: ignored_vars.clone(),
        };
        scope.push();
        for var in ignored_vars {
            scope.var_assign(
                var,
                true,
                IndexSet::from([GameValue::Anything]),
                VarSource::Ignore,
            );
        }
        scope
    }
    #[must_use]
    pub fn finish(mut self, check_child_scripts: bool, database: &Database) -> IndexSet<Issue> {
        self.pop();
        if check_child_scripts {
            let unused = &self.code_seen - &self.code_used;
            for expression in unused {
                let Expression::Code(statements) = expression else {
                    continue;
                };
                // trace!("-- Checking external scope");
                let mut external_scope =
                    Self::create(self.global.clone(), &self.ignored_vars, true);
                external_scope.eval_statements(&statements, database);
                self.errors
                    .extend(external_scope.finish(check_child_scripts, database));
            }
        }
        self.errors
    }

    pub fn push(&mut self) {
        // trace!("-- Stack Push {}", self.local.len());
        self.local.push(Stack::new());
    }
    pub fn pop(&mut self) {
        for (var, holder) in self.local.pop().unwrap_or_default() {
            // trace!("-- Stack Pop {}:{} ", var, holder.usage);
            if holder.usage == 0
                && !holder.source.skip_errors()
                && !self.ignored_vars.contains(&var)
            {
                self.errors.insert(Issue::Unused(var, holder.source));
            }
        }
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
            let mut global_m = self.global.borrow_mut();
            let holder = global_m.entry(var_lower).or_insert(VarHolder {
                possible: IndexSet::new(),
                usage: 0,
                source,
            });
            holder.possible.extend(possible_values);
            return;
        }

        let stack_level_search = self
            .local
            .iter()
            .rev()
            .position(|s| s.contains_key(&var_lower));
        let mut stack_level = self.local.len() - 1;
        if stack_level_search.is_none() {
            if !local {
                self.errors.insert(Issue::NotPrivate(
                    var.to_owned(),
                    source.get_range().unwrap_or_default(),
                ));
            }
        } else if local {
            // Only check shadowing inside the same scope-level (could make an option)
            if stack_level_search.unwrap_or_default() == 0 && !source.skip_errors() {
                self.errors.insert(Issue::Shadowed(
                    var.to_owned(),
                    source.get_range().unwrap_or_default(),
                ));
            }
        } else {
            stack_level -= stack_level_search.unwrap_or_default();
        }
        let holder = self.local[stack_level]
            .entry(var_lower)
            .or_insert(VarHolder {
                possible: IndexSet::new(),
                usage: 0,
                source,
            });
        holder.possible.extend(possible_values);
    }

    #[must_use]
    /// # Panics
    pub fn var_retrieve(
        &mut self,
        var: &str,
        source: &Range<usize>,
        peek: bool,
    ) -> IndexSet<GameValue> {
        let var_lower = var.to_ascii_lowercase();
        let mut global_m = self.global.borrow_mut();
        let holder_option = if var_lower.starts_with('_') {
            let stack_level_search = self
                .local
                .iter()
                .rev()
                .position(|s| s.contains_key(&var_lower));
            let mut stack_level = self.local.len() - 1;
            if let Some(stack_level_search) = stack_level_search {
                stack_level -= stack_level_search;
            } else if !peek {
                self.errors.insert(Issue::Undefined(
                    var.to_owned(),
                    source.clone(),
                    self.is_orphan_scope,
                ));
            }
            self.local[stack_level].get_mut(&var_lower)
        } else if global_m.contains_key(&var_lower) {
            global_m.get_mut(&var_lower)
        } else {
            return IndexSet::from([GameValue::Anything]);
        };
        if let Some(holder) = holder_option {
            holder.usage += 1;
            let mut set = holder.possible.clone();

            if !var_lower.starts_with('_') && self.ignored_vars.contains(&var.to_ascii_lowercase())
            {
                // Assume that a ignored global var could be anything
                set.insert(GameValue::Anything);
            }
            // println!("var_retrieve: `{var}` {set:?}");
            set
        } else {
            // we've reported the error above, just return Any so it doesn't fail everything after
            IndexSet::from([GameValue::Anything])
        }
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    /// Evaluate expression in current scope
    pub fn eval_expression(
        &mut self,
        expression: &Expression,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut debug_type = String::new();
        let possible_values = match expression {
            Expression::Variable(var, source) => self.var_retrieve(var, source, false),
            Expression::Number(..) => IndexSet::from([GameValue::Number(Some(expression.clone()))]),
            Expression::Boolean(..) => {
                IndexSet::from([GameValue::Boolean(Some(expression.clone()))])
            }
            Expression::String(..) => IndexSet::from([GameValue::String(Some(expression.clone()))]),
            Expression::Array(array, _) => {
                let gv_array: Vec<Vec<GameValue>> = array
                    .iter()
                    .map(|e| self.eval_expression(e, database).into_iter().collect())
                    .collect();
                IndexSet::from([GameValue::Array(Some(gv_array), None)])
            }
            Expression::NularCommand(cmd, source) => {
                debug_type = format!("[N:{}]", cmd.as_str());
                let mut cmd_set = GameValue::from_cmd(expression, None, None, database);
                if cmd_set.is_empty() {
                    // is this possible?
                    self.errors
                        .insert(Issue::InvalidArgs(debug_type.clone(), source.clone()));
                    cmd_set.insert(GameValue::Anything); // don't cause confusing errors for code downstream
                }
                cmd_set
            }
            Expression::UnaryCommand(cmd, rhs, source) => {
                debug_type = format!("[U:{}]", cmd.as_str());
                let rhs_set = self.eval_expression(rhs, database);
                let mut cmd_set = GameValue::from_cmd(expression, None, Some(&rhs_set), database);
                if cmd_set.is_empty() {
                    self.errors
                        .insert(Issue::InvalidArgs(debug_type.clone(), source.clone()));
                    cmd_set.insert(GameValue::Anything); // don't cause confusing errors for code downstream
                }
                let return_set = match cmd {
                    UnaryCommand::Named(named) => match named.to_ascii_lowercase().as_str() {
                        "params" => Some(self.cmd_generic_params(&rhs_set)),
                        "private" => Some(self.cmd_u_private(&rhs_set)),
                        "call" => Some(self.cmd_generic_call(&rhs_set, database)),
                        "isnil" => Some(self.cmd_u_is_nil(&rhs_set, database)),
                        "while" | "waituntil" | "default" => {
                            let _ = self.cmd_generic_call(&rhs_set, database);
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
                                        database,
                                    );
                                }
                            }
                            None
                        }
                        _ => None,
                    },
                    _ => None,
                };
                // Use custom return from cmd or just use wiki set
                return_set.unwrap_or(cmd_set)
            }
            Expression::BinaryCommand(cmd, lhs, rhs, source) => {
                debug_type = format!("[B:{}]", cmd.as_str());
                let lhs_set = self.eval_expression(lhs, database);
                let rhs_set = self.eval_expression(rhs, database);
                let mut cmd_set =
                    GameValue::from_cmd(expression, Some(&lhs_set), Some(&rhs_set), database);
                if cmd_set.is_empty() {
                    // we must have invalid args
                    self.errors
                        .insert(Issue::InvalidArgs(debug_type.clone(), source.clone()));
                    cmd_set.insert(GameValue::Anything); // don't cause confusing errors for code downstream
                }
                let return_set = match cmd {
                    BinaryCommand::Associate => {
                        // the : from case ToDo: these run outside of the do scope
                        let _ = self.cmd_generic_call(&rhs_set, database);
                        None
                    }
                    BinaryCommand::And | BinaryCommand::Or => {
                        let _ = self.cmd_generic_call(&rhs_set, database);
                        None
                    }
                    BinaryCommand::Else => Some(self.cmd_b_else(&lhs_set, &rhs_set)),
                    BinaryCommand::Eq => {
                        self.cmd_eqx_count_lint(lhs, rhs, database, true);
                        None
                    }
                    BinaryCommand::Greater | BinaryCommand::NotEq => {
                        self.cmd_eqx_count_lint(lhs, rhs, database, false);
                        None
                    }
                    BinaryCommand::Named(named) => match named.to_ascii_lowercase().as_str() {
                        "set" | "pushback" | "pushbackunique" | "append" | "resize" => {
                            // these commands modify the LHS by lvalue, assume it is now a generic array
                            self.cmd_generic_modify_lvalue(lhs);
                            None
                        }
                        "params" => Some(self.cmd_generic_params(&rhs_set)),
                        "call" => {
                            self.external_function(&lhs_set, rhs, database);
                            Some(self.cmd_generic_call(&rhs_set, database))
                        }
                        "exitwith" => {
                            // todo: handle scope exits
                            Some(self.cmd_generic_call(&rhs_set, database))
                        }
                        "do" => {
                            // from While, With, For, and Switch
                            // todo: handle switch return value
                            Some(self.cmd_b_do(&lhs_set, &rhs_set, database))
                        }
                        "from" | "to" | "step" => Some(self.cmd_b_from_chain(&lhs_set, &rhs_set)),
                        "then" => Some(self.cmd_b_then(&lhs_set, &rhs_set, database)),
                        "foreach" | "foreachreversed" => {
                            let mut magic = vec![
                                ("_x", GameValue::get_array_value_type(&rhs_set)),
                                ("_forEachIndex", GameValue::Number(None)),
                            ];
                            if !rhs_set.iter().all(|gv| matches!(gv, GameValue::Array(..))) {
                                magic.push(("_y", GameValue::Anything));
                            }
                            Some(self.cmd_generic_call_magic(&lhs_set, &magic, source, database))
                        }
                        "count" => {
                            let magic = vec![("_x", GameValue::get_array_value_type(&rhs_set))];
                            let _ = self.cmd_generic_call_magic(&lhs_set, &magic, source, database);
                            None
                        }
                        "apply" => {
                            let mut magic = vec![("_x", GameValue::get_array_value_type(&lhs_set))];
                            if !lhs_set.iter().all(|gv| matches!(gv, GameValue::Array(..))) {
                                magic.push(("_y", GameValue::Anything));
                            }
                            let _ = self.cmd_generic_call_magic(&rhs_set, &magic, source, database);
                            None
                        }
                        "findif" => {
                            let magic = vec![("_x", GameValue::get_array_value_type(&lhs_set))];
                            let _ = self.cmd_generic_call_magic(&rhs_set, &magic, source, database);
                            None
                        }
                        "getordefaultcall" => {
                            Some(self.cmd_b_get_or_default_call(&rhs_set, database))
                        }
                        "select" => {
                            Some(self.cmd_b_select(&lhs_set, &rhs_set, &cmd_set, source, database))
                        }
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
                                        database,
                                    );
                                }
                            }
                            None
                        }
                        _ => None,
                    },
                    _ => None,
                };
                // Use custom return from cmd or just use wiki set
                return_set.unwrap_or(cmd_set)
            }
            Expression::Code(statements) => {
                self.code_seen.insert(expression.clone());
                debug_type = format!("CODE:{}", statements.content().len());
                IndexSet::from([GameValue::Code(Some(expression.clone()))])
            }
            Expression::ConsumeableArray(_, _) => unreachable!(""),
        };
        trace!(
            "eval expression{}->{:?}",
            debug_type,
            possible_values
                .iter()
                .map(GameValue::as_debug)
                .collect::<Vec<_>>()
        );
        possible_values
    }

    /// Evaluate statements in the current scope
    fn eval_statements(&mut self, statements: &Statements, database: &Database) {
        // let mut return_value = IndexSet::new();
        for statement in statements.content() {
            match statement {
                Statement::AssignGlobal(var, expression, source) => {
                    // x or _x
                    let possible_values = self.eval_expression(expression, database);
                    self.var_assign(
                        var,
                        false,
                        possible_values,
                        VarSource::Assignment(
                            source.start..source.start + var.len(),
                            expression.span().clone(),
                        ),
                    );
                    // return_value = vec![GameValue::Assignment()];
                }
                Statement::AssignLocal(var, expression, source) => {
                    // private _x
                    let possible_values = self.eval_expression(expression, database);
                    self.var_assign(
                        var,
                        true,
                        possible_values,
                        VarSource::Assignment(
                            8 + source.start..8 + source.start + var.len(),
                            expression.span().clone(),
                        ),
                    );
                    // return_value = vec![GameValue::Assignment()];
                }
                Statement::Expression(expression, _) => {
                    let _possible_values = self.eval_expression(expression, database);
                    // return_value = possible_values;
                }
            }
        }
        // return_value
    }
}

#[must_use]
/// Run statements and return issues
pub fn run_processed(
    statements: &Statements,
    processed: &Processed,
    database: &Database,
) -> Vec<Issue> {
    let mut ignored_vars = IndexSet::new();
    ignored_vars.insert("_this".to_ascii_lowercase());
    ignored_vars.insert("_fnc_scriptName".to_ascii_lowercase()); // may be set via cfgFunctions
    ignored_vars.insert("_fnc_scriptNameParent".to_ascii_lowercase());
    let Ok(re1) =
        Regex::new(r"(?:\#pragma hemtt ignore_variables|\/\/ ?IGNORE_PRIVATE_WARNING) ?\[(.*)\]")
    else {
        return Vec::new();
    };
    let Ok(re2) = Regex::new(r#""(.*?)""#) else {
        return Vec::new();
    };
    for (_path, raw_source) in processed.sources() {
        for (_, [ignores]) in re1.captures_iter(&raw_source).map(|c| c.extract()) {
            for (_, [var]) in re2.captures_iter(ignores).map(|c| c.extract()) {
                ignored_vars.insert(var.to_ascii_lowercase());
            }
        }
    }

    let global = Rc::new(RefCell::new(Stack::new()));
    let mut scope = SciptScope::create(global, &ignored_vars, false);
    scope.eval_statements(statements, database);
    let rv: Vec<Issue> = scope.finish(true, database).into_iter().collect();
    // for ig in ignored_vars.clone() {
    //     if ig == "_this" {
    //         continue;
    //     }
    //     let mut igtest = ignored_vars.clone();
    //     igtest.shift_remove(&ig);

    //     let global = Rc::new(RefCell::new(Stack::new()));
    //     let mut scope = SciptScope::create(global, &igtest, false);
    //     scope.eval_statements(statements, database);
    //     let path = processed.sources();
    //     let new = scope.finish(true, database).len();
    //     if new <= rv.len() {
    //         println!(
    //             "in {:?}-{:?} and {} is undeeded [{}->{}]",
    //             path[0].0,
    //             path[path.len() - 1].0,
    //             ig,
    //             rv.len(),
    //             new
    //         );
    //     }
    // }
    rv
}
