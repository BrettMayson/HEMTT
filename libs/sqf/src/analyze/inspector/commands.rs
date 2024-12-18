//! Emulates engine commands

use std::{collections::HashSet, ops::Range};

use crate::{analyze::inspector::VarSource, parser::database::Database, Expression};

use super::{game_value::GameValue, SciptScope};

impl SciptScope {
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
                    VarSource::Private(source.clone()),
                );
            }
        }
        for possible in rhs {
            if let GameValue::Array(Some(gv_array)) = possible {
                for gv_index in gv_array {
                    for element in gv_index {
                        let GameValue::String(Some(Expression::String(var, source, _))) = element
                        else {
                            continue;
                        };
                        if var.is_empty() {
                            continue;
                        }
                        push_var(self, &var.to_string(), source);
                    }
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
            let GameValue::Array(Some(gv_array)) = possible else {
                continue;
            };

            for gv_index in gv_array {
                for element in gv_index {
                    match element {
                        GameValue::String(Some(Expression::String(var, source, _))) => {
                            if var.is_empty() {
                                continue;
                            }
                            self.var_assign(
                                var.as_ref(),
                                true,
                                HashSet::from([GameValue::Anything]),
                                VarSource::Params(source.clone()),
                            );
                        }
                        GameValue::Array(Some(arg_array)) => {
                            if arg_array.is_empty() || arg_array[0].is_empty() {
                                continue;
                            }
                            let GameValue::String(Some(Expression::String(var_name, source, _))) =
                                &arg_array[0][0]
                            else {
                                continue;
                            };
                            if var_name.is_empty() {
                                continue;
                            }
                            let mut var_types = HashSet::new();
                            if arg_array.len() > 2 {
                                for type_p in &arg_array[2] {
                                    if let GameValue::Array(Some(type_array)) = type_p {
                                        for type_i in type_array {
                                            var_types
                                                .extend(type_i.iter().map(GameValue::make_generic));
                                        }
                                    }
                                }
                            }
                            if var_types.is_empty() {
                                var_types.insert(GameValue::Anything);
                            }
                            // Add the default value to types
                            // It would be nice to move this above the is_empty check but not always safe
                            // ie: assume `params ["_z", ""]` is type string, but this is not guarentted
                            if arg_array.len() > 1 && !arg_array[1].is_empty() {
                                var_types.insert(arg_array[1][0].clone());
                            }
                            self.var_assign(
                                var_name.as_ref(),
                                true,
                                var_types,
                                VarSource::Params(source.clone()),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        HashSet::from([GameValue::Boolean(None)])
    }
    #[must_use]
    pub fn cmd_generic_call(
        &mut self,
        rhs: &HashSet<GameValue>,
        database: &Database,
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
            self.code_used.insert(expression.clone());
            self.eval_statements(statements, database);
            self.pop();
        }
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_b_do(
        &mut self,
        lhs: &HashSet<GameValue>,
        rhs: &HashSet<GameValue>,
        database: &Database,
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
            let mut do_run = false;
            for possible in lhs {
                if let GameValue::ForType(option) = possible {
                    let Some(for_args_array) = option else {
                        continue;
                    };
                    do_run = true;
                    for stage in for_args_array {
                        match stage {
                            Expression::String(var, source, _) => {
                                self.var_assign(
                                    var.as_ref(),
                                    true,
                                    HashSet::from([GameValue::Number(None)]),
                                    VarSource::ForLoop(source.clone()),
                                );
                            }
                            Expression::Code(stage_statement) => {
                                self.code_used.insert(stage.clone());
                                self.eval_statements(stage_statement, database);
                            }
                            _ => {}
                        }
                    }
                } else {
                    do_run = true;
                }
            }
            self.code_used.insert(expression.clone());
            if do_run {
                self.eval_statements(statements, database);
            }
            self.pop();
        }
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_generic_call_magic(
        &mut self,
        code_possibilities: &HashSet<GameValue>,
        magic: &Vec<(&str, GameValue)>,
        source: &Range<usize>,
        database: &Database,
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
            for (var, value) in magic {
                self.var_assign(
                    var,
                    true,
                    HashSet::from([value.clone()]),
                    VarSource::Magic(source.clone()),
                );
            }
            self.code_used.insert(expression.clone());
            self.eval_statements(statements, database);
            self.pop();
        }
        HashSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_for(&mut self, rhs: &HashSet<GameValue>) -> HashSet<GameValue> {
        let mut return_value = HashSet::new();
        for possible in rhs {
            let mut possible_array = Vec::new();
            match possible {
                GameValue::String(option) => {
                    let Some(expression) = option else {
                        return_value.insert(GameValue::ForType(None));
                        continue;
                    };
                    possible_array.push(expression.clone());
                }
                GameValue::Array(option) => {
                    let Some(for_stages) = option else {
                        return_value.insert(GameValue::ForType(None));
                        continue;
                    };
                    for stage in for_stages {
                        for gv in stage {
                            let GameValue::Code(Some(expression)) = gv else {
                                continue;
                            };
                            possible_array.push(expression.clone());
                        }
                    }
                }
                _ => {}
            }
            if possible_array.is_empty() {
                return_value.insert(GameValue::ForType(None));
            } else {
                return_value.insert(GameValue::ForType(Some(possible_array)));
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
    pub fn cmd_u_is_nil(
        &mut self,
        rhs: &HashSet<GameValue>,
        database: &Database,
    ) -> HashSet<GameValue> {
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
            let _ = self.cmd_generic_call(rhs, database);
        }
        HashSet::from([GameValue::Boolean(None)])
    }
    #[must_use]
    pub fn cmd_b_then(
        &mut self,
        _lhs: &HashSet<GameValue>,
        rhs: &HashSet<GameValue>,
        database: &Database,
    ) -> HashSet<GameValue> {
        let mut return_value = HashSet::new();
        for possible in rhs {
            if let GameValue::Code(Some(Expression::Code(_statements))) = possible {
                return_value.extend(self.cmd_generic_call(rhs, database));
            }
            if let GameValue::Array(Some(gv_array)) = possible {
                for gv_index in gv_array {
                    for element in gv_index {
                        if let GameValue::Code(Some(expression)) = element {
                            return_value.extend(self.cmd_generic_call(
                                &HashSet::from([GameValue::Code(Some(expression.clone()))]),
                                database,
                            ));
                        }
                    }
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
    pub fn cmd_b_get_or_default_call(
        &mut self,
        rhs: &HashSet<GameValue>,
        database: &Database,
    ) -> HashSet<GameValue> {
        let mut possible_code = HashSet::new();
        for possible_outer in rhs {
            let GameValue::Array(Some(gv_array)) = possible_outer else {
                continue;
            };
            if gv_array.len() < 2 {
                continue;
            }
            possible_code.extend(gv_array[1].clone());
        }
        let _ = self.cmd_generic_call(&possible_code, database);
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
    pub fn cmd_b_select(
        &mut self,
        lhs: &HashSet<GameValue>,
        rhs: &HashSet<GameValue>,
        cmd_set: &HashSet<GameValue>,
        source: &Range<usize>,
        database: &Database,
    ) -> HashSet<GameValue> {
        let mut return_value = cmd_set.clone();
        // Check: `array select expression`
        let _ =
            self.cmd_generic_call_magic(rhs, &vec![("_x", GameValue::Anything)], source, database);
        // if lhs is array, and rhs is bool/number then put array into return
        if lhs.len() == 1
            && rhs
                .iter()
                .any(|r| matches!(r, GameValue::Boolean(..)) || matches!(r, GameValue::Number(..)))
        {
            if let Some(GameValue::Array(Some(gv_array))) = lhs.iter().next() {
                // return_value.clear(); // todo: could clear if we handle pushBack
                for gv_index in gv_array {
                    for element in gv_index {
                        return_value.insert(element.clone());
                    }
                }
            }
        }
        return_value
    }
}
