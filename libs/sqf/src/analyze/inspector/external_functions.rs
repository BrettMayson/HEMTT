use std::collections::HashSet;

use crate::{analyze::inspector::VarSource, Expression};

use super::{game_value::GameValue, SciptScope};

impl SciptScope {
    pub fn external_function(&mut self, lhs: &HashSet<GameValue>, rhs: &Expression) {
        let Expression::Variable(ext_func, _) = rhs else {
            return;
        };
        for possible in lhs {
            let GameValue::Array(Some(gv_array)) = possible else {
                continue;
            };
            match ext_func.to_ascii_lowercase().as_str() {
                "cba_fnc_hasheachpair" | "cba_fnc_hashfilter" => {
                    if gv_array.len() > 1 {
                        self.external_current_scope(
                            &gv_array[1],
                            &vec![
                                ("_key", GameValue::Anything),
                                ("_value", GameValue::Anything),
                            ],
                        );
                    }
                }
                "cba_fnc_filter" => {
                    if gv_array.len() > 1 {
                        self.external_current_scope(
                            &gv_array[1],
                            &vec![("_x", GameValue::Anything)],
                        );
                    }
                }
                "cba_fnc_directcall" => {
                    if !gv_array.is_empty() {
                        self.external_current_scope(&gv_array[0], &vec![]);
                    }
                }
                "ace_common_fnc_cachedcall" => {
                    if gv_array.len() > 1 {
                        self.external_current_scope(&gv_array[1], &vec![]);
                    }
                }
                "ace_interact_menu_fnc_createaction" => {
                    for index in 3..5 {
                        if gv_array.len() > index {
                            self.external_new_scope(
                                &gv_array[index],
                                &vec![
                                    ("_target", GameValue::Object),
                                    ("_player", GameValue::Object),
                                ],
                            );
                        }
                    }
                }
                "cba_fnc_addperframehandler" | "cba_fnc_waitandexecute" => {
                    if !gv_array.is_empty() {
                        self.external_new_scope(&gv_array[0], &vec![]);
                    }
                }
                "cba_fnc_addclasseventhandler" => {
                    if gv_array.len() > 2 {
                        self.external_new_scope(&gv_array[2], &vec![]);
                    }
                }
                _ => {}
            }
        }
    }
    fn external_new_scope(&mut self, code_arg: &Vec<GameValue>, vars: &Vec<(&str, GameValue)>) {
        for element in code_arg {
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
    fn external_current_scope(&mut self, code_arg: &Vec<GameValue>, vars: &Vec<(&str, GameValue)>) {
        for element in code_arg {
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
