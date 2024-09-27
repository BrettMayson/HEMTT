use std::collections::HashSet;

use crate::{analyze::inspector::VarSource, Expression};

use super::{game_value::GameValue, SciptScope};

impl SciptScope {
    pub fn external_function(&mut self, lhs: &HashSet<GameValue>, rhs: &Expression) {
        let Expression::Variable(var, _) = rhs else {
            return;
        };
        for possible in lhs {
            let GameValue::Array(Some(Expression::Array(array, _))) = possible else {
                continue;
            };
            match var.to_ascii_lowercase().as_str() {
                "cba_fnc_hasheachpair" | "cba_fnc_hashfilter" => {
                    if array.len() > 1 {
                        let code = self.eval_expression(&array[1]);
                        self.external_current_scope(
                            &code,
                            &vec![
                                ("_key", GameValue::Anything),
                                ("_value", GameValue::Anything),
                            ],
                        );
                    }
                }
                "cba_fnc_filter" => {
                    if array.len() > 1 {
                        let code = self.eval_expression(&array[1]);
                        self.external_current_scope(&code, &vec![("_x", GameValue::Anything)]);
                    }
                }
                "cba_fnc_directcall" => {
                    if !array.is_empty() {
                        let code = self.eval_expression(&array[0]);
                        self.external_current_scope(&code, &vec![]);
                    }
                }
                "ace_interact_menu_fnc_createaction" => {
                    for index in 3..5 {
                        if array.len() > index {
                            let code = self.eval_expression(&array[index]);
                            self.external_new_scope(
                                &code,
                                &vec![
                                    ("_target", GameValue::Object),
                                    ("_player", GameValue::Object),
                                ],
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn external_new_scope(
        &mut self,
        possible_arg: &HashSet<GameValue>,
        vars: &Vec<(&str, GameValue)>,
    ) {
        for element in possible_arg {
            let GameValue::Code(Some(expression)) = element else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                return;
            };
            if self.code_used.contains(expression) {
                return;
            }
            let mut ext_scope = Self::create(&self.ignored_vars, &self.database, true);

            for (var, value) in vars {
                ext_scope.var_assign(var, true, HashSet::from([value.clone()]), VarSource::Ignore);
            }
            self.code_used.insert(expression.clone());
            ext_scope.eval_statements(statements);
            self.errors.extend(ext_scope.finish(false));
        }
    }
    fn external_current_scope(
        &mut self,
        possible_arg: &HashSet<GameValue>,
        vars: &Vec<(&str, GameValue)>,
    ) {
        for element in possible_arg {
            let GameValue::Code(Some(expression)) = element else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            if self.code_used.contains(expression) {
                continue;
            }
            self.push();
            for (var, value) in vars {
                self.var_assign(var, true, HashSet::from([value.clone()]), VarSource::Ignore);
            }
            self.code_used.insert(expression.clone());
            self.eval_statements(statements);
            self.pop();
        }
    }
}
