//! Inspects code, checking code args and variable usage
//!
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::Range,
    sync::Arc,
    vec,
};

use crate::{
    parser::database::Database, BinaryCommand, Expression, Statement, Statements, UnaryCommand,
};
use game_value::GameValue;
use hemtt_workspace::reporting::Processed;
use regex::Regex;
use tracing::{error, trace};

mod game_value;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Issue {
    InvalidArgs(String, Range<usize>),
    Undefined(String, Range<usize>, bool),
    Unused(String, Range<usize>),
    Shadowed(String, Range<usize>),
    NotPrivate(String, Range<usize>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VarSource {
    Real(Range<usize>),
    Magic(Range<usize>),
    Ignore,
}
impl VarSource {
    #[must_use]
    pub const fn check_unused(&self) -> bool {
        matches!(self, Self::Real(..))
    }
    #[must_use]
    pub const fn check_shadow(&self) -> bool {
        matches!(self, Self::Real(..))
    }
    #[must_use]
    pub fn get_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Real(range) | Self::Magic(range) => Some(range.clone()),
            Self::Ignore => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarHolder {
    possible: HashSet<GameValue>,
    usage: i32,
    source: VarSource,
}

pub type Stack = HashMap<String, VarHolder>;

pub struct SciptScope {
    database: Arc<Database>,
    errors: HashSet<Issue>,
    global: Stack,
    local: Vec<Stack>,
    code_seen: HashSet<Expression>,
    code_used: HashSet<Expression>,
    is_child: bool,
    ignored_vars: HashSet<String>,
}

impl SciptScope {
    #[must_use]
    pub fn create(
        ignored_vars: &HashSet<String>,
        database: &Arc<Database>,
        is_child: bool,
    ) -> Self {
        // trace!("Creating ScriptScope");
        let mut scope = Self {
            database: database.clone(),
            errors: HashSet::new(),
            global: Stack::new(),
            local: Vec::new(),
            code_seen: HashSet::new(),
            code_used: HashSet::new(),
            is_child,
            ignored_vars: ignored_vars.clone(),
        };
        scope.push();
        for var in ignored_vars {
            scope.var_assign(
                var,
                true,
                HashSet::from([GameValue::Anything]),
                VarSource::Ignore,
            );
        }
        scope
    }
    #[must_use]
    pub fn finish(&mut self, check_child_scripts: bool) -> HashSet<Issue> {
        self.pop();
        if check_child_scripts {
            let unused = &self.code_seen - &self.code_used;
            for expression in unused {
                let Expression::Code(statements) = expression else {
                    error!("non-code in unused");
                    continue;
                };
                // trace!("-- Checking external scope");
                let mut external_scope = Self::create(&self.ignored_vars, &self.database, true);
                external_scope.eval_statements(&statements);
                self.errors
                    .extend(external_scope.finish(check_child_scripts));
            }
        }
        self.errors.clone()
    }

    pub fn push(&mut self) {
        // trace!("-- Stack Push {}", self.local.len());
        self.local.push(Stack::new());
    }
    pub fn pop(&mut self) {
        for (var, holder) in self.local.pop().unwrap_or_default() {
            // trace!("-- Stack Pop {}:{} ", var, holder.usage);
            if holder.usage == 0 && holder.source.check_unused() {
                self.errors.insert(Issue::Unused(
                    var,
                    holder.source.get_range().unwrap_or_default(),
                ));
            }
        }
    }

    pub fn var_assign(
        &mut self,
        var: &str,
        local: bool,
        possible_values: HashSet<GameValue>,
        source: VarSource,
    ) {
        trace!("var_assign: {} @ {}", var, self.local.len());
        let var_lower = var.to_ascii_lowercase();
        if !var_lower.starts_with('_') {
            let holder = self.global.entry(var_lower).or_insert(VarHolder {
                possible: HashSet::new(),
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
            if source.check_shadow() {
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
                possible: HashSet::new(),
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
    ) -> HashSet<GameValue> {
        let var_lower = var.to_ascii_lowercase();
        let holder_option = if var_lower.starts_with('_') {
            let stack_level_search = self
                .local
                .iter()
                .rev()
                .position(|s| s.contains_key(&var_lower));
            let mut stack_level = self.local.len() - 1;
            if stack_level_search.is_none() {
                if !peek {
                    self.errors.insert(Issue::Undefined(
                        var.to_owned(),
                        source.clone(),
                        self.is_child,
                    ));
                }
            } else {
                stack_level -= stack_level_search.expect("is_some");
            };
            self.local[stack_level].get_mut(&var_lower)
        } else if self.global.contains_key(&var_lower) {
            self.global.get_mut(&var_lower)
        } else {
            return HashSet::from([GameValue::Anything]);
        };
        if holder_option.is_none() {
            // we've reported the error above, just return Any so it doesn't fail everything after
            HashSet::from([GameValue::Anything])
        } else {
            let holder = holder_option.expect("is_some");
            holder.usage += 1;
            let mut set = holder.possible.clone();

            if !var_lower.starts_with('_') && self.ignored_vars.contains(&var.to_ascii_lowercase())
            {
                // Assume that a ignored global var could be anything
                set.insert(GameValue::Anything);
            }
            set
        }
    }
    #[must_use]
    pub fn cmd_u_private(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        fn push_var(s: &mut SciptScope, var: &String, source: &Range<usize>) {
            if s.ignored_vars.contains(&var.to_ascii_lowercase()) {
                s.var_assign(
                    &var.to_string(),
                    true,
                    HashSet::from([GameValue::Anything]),
                    VarSource::Ignore,
                );
            } else {
                s.var_assign(
                    &var.to_string(),
                    true,
                    HashSet::from([GameValue::Nothing]),
                    VarSource::Real(source.clone()),
                );
            }
        }
        for possible in rhs {
            if let GameValue::Array(Some(Expression::Array(array, _))) = possible {
                for element in array {
                    let Expression::String(var, source, _) = element else {
                        continue;
                    };
                    if var.is_empty() {
                        continue;
                    }
                    push_var(self, &var.to_string(), source);
                }
            }
            if let GameValue::String(Some(Expression::String(var, source, _))) = possible {
                if var.is_empty() {
                    continue;
                }
                push_var(self, &var.to_string(), source);
            }
        }
        HashSet::new()
    }
    #[must_use]
    pub fn cmd_generic_params(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        for possible in rhs {
            let GameValue::Array(Some(Expression::Array(array, _))) = possible else {
                continue;
            };

            for entry in array {
                match entry {
                    Expression::String(var, source, _) => {
                        if var.is_empty() {
                            continue;
                        }
                        self.var_assign(
                            var.as_ref(),
                            true,
                            HashSet::from([GameValue::Anything]),
                            VarSource::Real(source.clone()),
                        );
                    }
                    Expression::Array(var_array, _) => {
                        if !var_array.is_empty() {
                            if let Expression::String(var, source, _) = &var_array[0] {
                                if var.is_empty() {
                                    continue;
                                }
                                self.var_assign(
                                    var.as_ref(),
                                    true,
                                    HashSet::from([GameValue::Anything]),
                                    VarSource::Real(source.clone()),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        HashSet::from([GameValue::Boolean(None)])
    }
    #[must_use]
    pub fn cmd_generic_call(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        for possible in rhs {
            let GameValue::Code(Some(expression)) = possible else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            if self.code_used.contains(expression) {
                continue;
            }
            self.push();
            self.code_used.insert(expression.clone());
            self.eval_statements(statements);
            self.pop();
        }
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_b_do(
        &mut self,
        lhs: &HashSet<GameValue>,
        rhs: &HashSet<GameValue>,
    ) -> HashSet<GameValue> {
        for possible in rhs {
            let GameValue::Code(Some(expression)) = possible else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            if self.code_used.contains(expression) {
                continue;
            }
            self.push();
            // look for forType vars with valid strings (ignore old style code)
            let mut do_run = true;
            for possible in lhs {
                if let GameValue::ForType(option) = possible {
                    match option {
                        Some(Expression::String(var, source, _)) => {
                            self.var_assign(
                                var.as_ref(),
                                true,
                                HashSet::from([GameValue::Number(None)]),
                                VarSource::Real(source.clone()),
                            );
                        }
                        Some(Expression::Array(array, _)) => {
                            if array.len() != 3 {
                                error!("for wrong len");
                                continue;
                            }
                            for for_stage in array {
                                let Expression::Code(for_statements) = for_stage else {
                                    continue;
                                };
                                self.code_used.insert(for_stage.clone());
                                self.eval_statements(for_statements);
                            }
                        }
                        None => {
                            do_run = false;
                        }
                        _ => {
                            unreachable!("");
                        }
                    }
                }
            }
            self.code_used.insert(expression.clone());
            if do_run {
                self.eval_statements(statements);
            }
            self.pop();
        }
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_generic_call_magic(
        &mut self,
        code_possibilities: &HashSet<GameValue>,
        magic: &Vec<String>,
        source: &Range<usize>,
    ) -> HashSet<GameValue> {
        for possible in code_possibilities {
            let GameValue::Code(Some(expression)) = possible else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            if self.code_used.contains(expression) {
                continue;
            }
            self.push();
            for var in magic {
                self.var_assign(
                    var,
                    true,
                    HashSet::from([GameValue::Anything]),
                    VarSource::Magic(source.clone()),
                );
            }
            self.code_used.insert(expression.clone());
            self.eval_statements(statements);
            self.pop();
        }
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_for(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        let mut return_value = HashSet::new();
        for possible in rhs {
            match possible {
                GameValue::Array(option) | GameValue::String(option) => {
                    return_value.insert(GameValue::ForType(option.clone()));
                }
                _ => {
                    error!("shouldn't be reachable?");
                    return_value.insert(GameValue::ForType(None));
                }
            }
        }
        return_value
    }
    #[must_use]
    /// for (from, to, step) chained commands
    pub fn cmd_b_from_chain(
        &self,
        lhs: &HashSet<GameValue>,
        _rhs: &HashSet<GameValue>,
    ) -> HashSet<GameValue> {
        lhs.clone()
    }
    #[must_use]
    pub fn cmd_u_is_nil(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        let mut non_string = false;
        for possible in rhs {
            let GameValue::String(possible) = possible else {
                non_string = true;
                continue;
            };
            let Some(expression) = possible else {
                continue;
            };
            let Expression::String(var, _, _) = expression else {
                continue;
            };
            let _ = self.var_retrieve(var, &expression.span(), true);
        }
        if non_string {
            let _ = self.cmd_generic_call(rhs);
        }
        HashSet::from([GameValue::Boolean(None)])
    }
    #[must_use]
    pub fn cmd_b_then(
        &mut self,
        _lhs: &HashSet<GameValue>,
        rhs: &HashSet<GameValue>,
    ) -> HashSet<GameValue> {
        let mut return_value = HashSet::new();
        for possible in rhs {
            if let GameValue::Code(Some(Expression::Code(_statements))) = possible {
                return_value.extend(self.cmd_generic_call(rhs));
            }
            if let GameValue::Array(Some(Expression::Array(array, _))) = possible {
                for expression in array {
                    return_value.extend(self.cmd_generic_call(&HashSet::from([GameValue::Code(
                        Some(expression.clone()),
                    )])));
                }
            }
        }
        return_value
    }
    #[must_use]
    pub fn cmd_b_else(
        &self,
        lhs: &HashSet<GameValue>,
        rhs: &HashSet<GameValue>,
    ) -> HashSet<GameValue> {
        let mut return_value = HashSet::new(); // just merge, not really the same but should be fine
        for possible in rhs {
            return_value.insert(possible.clone());
        }
        for possible in lhs {
            return_value.insert(possible.clone());
        }
        return_value
    }
    #[must_use]
    pub fn cmd_b_get_or_default_call(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        let mut possible_code = HashSet::new();
        for possible in rhs {
            let GameValue::Array(Some(Expression::Array(array, _))) = possible else {
                continue;
            };
            if array.len() < 2 {
                continue;
            }
            possible_code.insert(GameValue::Code(Some(array[1].clone())));
        }
        let _ = self.cmd_generic_call(&possible_code);
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_u_to_string(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        for possible in rhs {
            let GameValue::Code(Some(expression)) = possible else {
                continue;
            };
            let Expression::Code(_) = expression else {
                continue;
            };
            // just skip because it will often use a _x
            self.code_used.insert(expression.clone());
        }
        HashSet::from([GameValue::String(None)])
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    /// Evaluate expression in current scope
    pub fn eval_expression(&mut self, expression: &Expression) -> HashSet<GameValue> {
        let mut debug_type = String::new();
        let possible_values = match expression {
            Expression::Variable(var, source) => self.var_retrieve(var, source, false),
            Expression::Number(..) => HashSet::from([GameValue::Number(Some(expression.clone()))]),
            Expression::Boolean(..) => {
                HashSet::from([GameValue::Boolean(Some(expression.clone()))])
            }
            Expression::String(..) => HashSet::from([GameValue::String(Some(expression.clone()))]),
            Expression::Array(array, _) => {
                for e in array {
                    let _ = self.eval_expression(e);
                }
                HashSet::from([GameValue::Array(Some(expression.clone()))])
            }
            Expression::NularCommand(cmd, source) => {
                debug_type = format!("[N:{}]", cmd.as_str());
                let cmd_set = GameValue::from_cmd(expression, None, None, &self.database);
                if cmd_set.is_empty() {
                    // is this possible?
                    self.errors
                        .insert(Issue::InvalidArgs(debug_type.clone(), source.clone()));
                }
                cmd_set
            }
            Expression::UnaryCommand(cmd, rhs, source) => {
                debug_type = format!("[U:{}]", cmd.as_str());
                let rhs_set = self.eval_expression(rhs);
                let cmd_set = GameValue::from_cmd(expression, None, Some(&rhs_set), &self.database);
                if cmd_set.is_empty() {
                    self.errors
                        .insert(Issue::InvalidArgs(debug_type.clone(), source.clone()));
                }
                let return_set = match cmd {
                    UnaryCommand::Named(named) => match named.to_ascii_lowercase().as_str() {
                        "params" => Some(self.cmd_generic_params(&rhs_set)),
                        "private" => Some(self.cmd_u_private(&rhs_set)),
                        "call" => Some(self.cmd_generic_call(&rhs_set)),
                        "isnil" => Some(self.cmd_u_is_nil(&rhs_set)),
                        "while" | "waituntil" | "default" => {
                            let _ = self.cmd_generic_call(&rhs_set);
                            None
                        }
                        "for" => Some(self.cmd_for(&rhs_set)),
                        "tostring" => Some(self.cmd_u_to_string(&rhs_set)),
                        _ => None,
                    },
                    _ => None,
                };
                // Use custom return from cmd or just use wiki set
                return_set.unwrap_or(cmd_set)
            }
            Expression::BinaryCommand(cmd, lhs, rhs, source) => {
                debug_type = format!("[B:{}]", cmd.as_str());
                let lhs_set = self.eval_expression(lhs);
                let rhs_set = self.eval_expression(rhs);
                let cmd_set =
                    GameValue::from_cmd(expression, Some(&lhs_set), Some(&rhs_set), &self.database);
                if cmd_set.is_empty() {
                    // we must have invalid args
                    self.errors
                        .insert(Issue::InvalidArgs(debug_type.clone(), source.clone()));
                }
                let return_set = match cmd {
                    BinaryCommand::Associate => {
                        // the : from case
                        let _ = self.cmd_generic_call(&rhs_set);
                        None
                    }
                    BinaryCommand::And | BinaryCommand::Or => {
                        let _ = self.cmd_generic_call(&rhs_set);
                        None
                    }
                    BinaryCommand::Else => Some(self.cmd_b_else(&lhs_set, &rhs_set)),
                    BinaryCommand::Named(named) => match named.to_ascii_lowercase().as_str() {
                        "params" => Some(self.cmd_generic_params(&rhs_set)),
                        "call" => Some(self.cmd_generic_call(&rhs_set)),
                        "exitwith" => {
                            // todo: handle scope exits
                            Some(self.cmd_generic_call(&rhs_set))
                        }
                        "do" => {
                            // from While, With, For, and Switch
                            Some(self.cmd_b_do(&lhs_set, &rhs_set))
                        }
                        "from" | "to" | "step" => Some(self.cmd_b_from_chain(&lhs_set, &rhs_set)),
                        "then" => Some(self.cmd_b_then(&lhs_set, &rhs_set)),
                        "foreach" | "foreachreversed" => Some(self.cmd_generic_call_magic(
                            &lhs_set,
                            &vec![
                                "_x".to_string(),
                                "_y".to_string(),
                                "_forEachIndex".to_string(),
                            ],
                            source,
                        )),
                        "count" => {
                            let _ = self.cmd_generic_call_magic(
                                &lhs_set,
                                &vec!["_x".to_string()],
                                source,
                            );
                            None
                        }
                        "findif" | "apply" | "select" => {
                            //todo handle (array select number) or (string select [1,2]);
                            let _ = self.cmd_generic_call_magic(
                                &rhs_set,
                                &vec!["_x".to_string()],
                                source,
                            );
                            None
                        }
                        "getordefaultcall" => Some(self.cmd_b_get_or_default_call(&rhs_set)),
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
                HashSet::from([GameValue::Code(Some(expression.clone()))])
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
    fn eval_statements(&mut self, statements: &Statements) {
        // let mut return_value = HashSet::new();
        for statement in statements.content() {
            match statement {
                Statement::AssignGlobal(var, expression, source) => {
                    // x or _x
                    let possible_values = self.eval_expression(expression);
                    self.var_assign(var, false, possible_values, VarSource::Real(source.clone()));
                    // return_value = vec![GameValue::Assignment()];
                }
                Statement::AssignLocal(var, expression, source) => {
                    // private _x
                    let possible_values = self.eval_expression(expression);
                    self.var_assign(var, true, possible_values, VarSource::Real(source.clone()));
                    // return_value = vec![GameValue::Assignment()];
                }
                Statement::Expression(expression, _) => {
                    let _possible_values = self.eval_expression(expression);
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
    database: &Arc<Database>,
    check_child_scripts: bool,
) -> Vec<Issue> {
    let mut ignored_vars = Vec::new();
    ignored_vars.push("_this".to_string());
    let Ok(re1) = Regex::new(r"\/\/ ?IGNORE_PRIVATE_WARNING ?\[(.*)\]") else {
        return Vec::new();
    };
    let Ok(re2) = Regex::new(r#""(.*?)""#) else {
        return Vec::new();
    };
    for (_path, raw_source) in processed.sources() {
        for (_, [ignores]) in re1.captures_iter(&raw_source).map(|c| c.extract()) {
            for (_, [var]) in re2.captures_iter(ignores).map(|c| c.extract()) {
                ignored_vars.push(var.to_ascii_lowercase());
            }
        }
    }
    let mut scope = SciptScope::create(&HashSet::from_iter(ignored_vars), database, false);
    scope.eval_statements(statements);
    scope.finish(check_child_scripts).into_iter().collect()
}
