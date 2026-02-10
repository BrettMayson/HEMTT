//! Emulate how common external functions will handle code

use std::ops::Range;

use indexmap::IndexSet;

use crate::{Expression, analyze::inspector::VarSource};

use super::{Inspector, game_value::GameValue};

impl Inspector<'_> {
    pub fn external_function(&mut self, lhs: &IndexSet<GameValue>, rhs: &Expression) {
        let Expression::Variable(ext_func, _) = rhs else {
            return;
        };
        let ext_func_lower = ext_func.to_ascii_lowercase();
        for possible in lhs {
            match possible {
                GameValue::Code(Some(statements)) => {
                    // handle `{} call cba_fnc_directcall`
                    if ext_func_lower.as_str() == "cba_fnc_directcall" {
                        self.external_current_scope(
                            &vec![(GameValue::Code(Some(statements.clone())), statements.span())],
                            &vec![],
                        );
                    }
                }
                GameValue::Array(Some(gv_array), _) => match ext_func_lower.as_str() {
                    // Functions that will run in existing scope
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
                    "cba_fnc_inject" => {
                        if gv_array.len() > 2 {
                            self.external_current_scope(
                                &gv_array[2],
                                &vec![
                                    ("_x", GameValue::Anything),
                                    ("_accumulator", GameValue::Anything),
                                ],
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
                    // Functions that will start in a new scope
                    "ace_interact_menu_fnc_createaction" => {
                        for index in 3..=5 {
                            if gv_array.len() > index {
                                self.external_new_scope(
                                    &gv_array[index],
                                    &vec![
                                        ("_target", GameValue::Object),
                                        ("_player", GameValue::Object),
                                        ("_actionParams", GameValue::Anything),
                                    ],
                                );
                            }
                        }
                    }
                    "cba_fnc_addperframehandler"
                    | "cba_fnc_waitandexecute"
                    | "cba_fnc_execnextframe" => {
                        if !gv_array.is_empty() {
                            self.external_new_scope(&gv_array[0], &vec![]);
                        }
                    }
                    "cba_fnc_addclasseventhandler" => {
                        if gv_array.len() > 2 {
                            self.external_new_scope(&gv_array[2], &vec![]);
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
                            );
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    pub fn external_new_scope(
        &mut self,
        code_arg: &Vec<(GameValue, Range<usize>)>,
        vars: &Vec<(&str, GameValue)>,
    ) {
        for (element, _) in code_arg {
            let GameValue::Code(Some(expression)) = element else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            self.scope_push(false);
            let stack_index = self.stack_push(Some(expression), false);
            if stack_index.is_some() {
                // prevent infinite recursion
                for (var, value) in vars {
                    self.var_assign(
                        var,
                        true,
                        IndexSet::from([value.clone()]),
                        VarSource::Ignore,
                    );
                }
                self.eval_statements(statements, false);
                let _ = self.stack_pop(stack_index);
            }
            self.scope_pop();
        }
    }
    fn external_current_scope(
        &mut self,
        code_arg: &Vec<(GameValue, Range<usize>)>,
        vars: &Vec<(&str, GameValue)>,
    ) {
        for (element, _) in code_arg {
            let GameValue::Code(Some(expression)) = element else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            let stack_index = self.stack_push(Some(expression), false);
            if stack_index.is_none() {
                continue;
            }
            for (var, value) in vars {
                self.var_assign(
                    var,
                    true,
                    IndexSet::from([value.clone()]),
                    VarSource::Ignore,
                );
            }
            self.eval_statements(statements, true);
            self.stack_pop(stack_index);
        }
    }
}
