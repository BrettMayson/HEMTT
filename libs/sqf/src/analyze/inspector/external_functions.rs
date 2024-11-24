//! Emulate how common external functions will handle code

use std::collections::HashSet;

use crate::{analyze::inspector::VarSource, parser::database::Database, Expression};

use super::{game_value::GameValue, SciptScope};

impl SciptScope {
    #[allow(clippy::too_many_lines)]
    pub fn external_function(
        &mut self,
        lhs: &HashSet<GameValue>,
        rhs: &Expression,
        database: &Database,
    ) {
        let Expression::Variable(ext_func, _) = rhs else {
            return;
        };
        for possible in lhs {
            match possible {
                GameValue::Code(Some(statements)) => {
                    // handle `{} call cba_fnc_directcall`
                    if ext_func.to_ascii_lowercase().as_str() == "cba_fnc_directcall" {
                        self.external_current_scope(
                            &vec![GameValue::Code(Some(statements.clone()))],
                            &vec![],
                            database,
                        );
                    }
                }
                GameValue::Array(Some(gv_array)) => match ext_func.to_ascii_lowercase().as_str() {
                    // Functions that will run in existing scope
                    "cba_fnc_hasheachpair" | "cba_fnc_hashfilter" => {
                        if gv_array.len() > 1 {
                            self.external_current_scope(
                                &gv_array[1],
                                &vec![
                                    ("_key", GameValue::Anything),
                                    ("_value", GameValue::Anything),
                                ],
                                database,
                            );
                        }
                    }
                    "cba_fnc_filter" => {
                        if gv_array.len() > 1 {
                            self.external_current_scope(
                                &gv_array[1],
                                &vec![("_x", GameValue::Anything)],
                                database,
                            );
                        }
                    }
                    "cba_fnc_inject" => {
                        if gv_array.len() > 2 {
                            self.external_current_scope(
                                &gv_array[2],
                                &vec![
                                    ("_x", GameValue::Anything),
                                    ("_accumulator", GameValue::Anything),
                                ],
                                database,
                            );
                        }
                    }
                    "cba_fnc_directcall" => {
                        if !gv_array.is_empty() {
                            self.external_current_scope(&gv_array[0], &vec![], database);
                        }
                    }
                    "ace_common_fnc_cachedcall" => {
                        if gv_array.len() > 1 {
                            self.external_current_scope(&gv_array[1], &vec![], database);
                        }
                    }
                    // Functions that will start in a new scope
                    "ace_interact_menu_fnc_createaction" => {
                        for index in 3..=5 {
                            if gv_array.len() > index {
                                self.external_new_scope(
                                    &gv_array[index],
                                    &vec![
                                        ("_target", GameValue::Object),
                                        ("_player", GameValue::Object),
                                    ],
                                    database,
                                );
                            }
                        }
                    }
                    "cba_fnc_addperframehandler" | "cba_fnc_waitandexecute" | "cba_fnc_execnextframe" => {
                        if !gv_array.is_empty() {
                            self.external_new_scope(&gv_array[0], &vec![], database);
                        }
                    }
                    "cba_fnc_addclasseventhandler" => {
                        if gv_array.len() > 2 {
                            self.external_new_scope(&gv_array[2], &vec![], database);
                        }
                    }
                    "cba_fnc_addbiseventhandler" => {
                        if gv_array.len() > 2 {
                            self.external_new_scope(
                                &gv_array[2],
                                &vec![
                                    ("_thisType", GameValue::String(None)),
                                    ("_thisId", GameValue::Number(None)),
                                    ("_thisFnc", GameValue::Code(None)),
                                    ("_thisArgs", GameValue::Anything),
                                ],
                                database,
                            );
                        }
                    }
                    "cba_fnc_addeventhandlerargs" => {
                        if gv_array.len() > 1 {
                            self.external_new_scope(
                                &gv_array[1],
                                &vec![
                                    ("_thisType", GameValue::String(None)),
                                    ("_thisId", GameValue::Number(None)),
                                    ("_thisFnc", GameValue::Code(None)),
                                    ("_thisArgs", GameValue::Anything),
                                ],
                                database,
                            );
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    fn external_new_scope(
        &mut self,
        code_arg: &Vec<GameValue>,
        vars: &Vec<(&str, GameValue)>,
        database: &Database,
    ) {
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
            let mut ext_scope = Self::create(&self.ignored_vars, false);

            for (var, value) in vars {
                ext_scope.var_assign(var, true, HashSet::from([value.clone()]), VarSource::Ignore);
            }
            self.code_used.insert(expression.clone());
            ext_scope.eval_statements(statements, database);
            self.errors.extend(ext_scope.finish(false, database));
        }
    }
    fn external_current_scope(
        &mut self,
        code_arg: &Vec<GameValue>,
        vars: &Vec<(&str, GameValue)>,
        database: &Database,
    ) {
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
            self.eval_statements(statements, database);
            self.pop();
        }
    }
}
