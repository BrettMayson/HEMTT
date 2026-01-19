//! Emulate how common external functions will handle code

use std::ops::Range;

use arma3_wiki::model::Arg;
use indexmap::IndexSet;
#[allow(unused_imports)]
use tracing::{info, trace, warn};

use crate::{
    Expression,
    analyze::inspector::{InvalidArgs, Issue, VarSource, game_value::NilSource},
    parser::database::Database,
};

use super::{Inspector, game_value::GameValue};

impl Inspector {
    /// Analyze external function calls in database, checking parameters and getting return type
    #[must_use]
    pub fn external_function_call(
        &mut self,
        lhs: Option<&IndexSet<GameValue>>,
        rhs: &Expression,
        database: &Database,
    ) -> Option<IndexSet<GameValue>> {
        let Expression::Variable(ext_func, span) = rhs else {
            return None;
        };
        if ext_func.starts_with('_') {
            return None;
        }
        let ext_func_lower = ext_func.to_ascii_lowercase();

        if let Some(lhs) = lhs {
            self.external_check_code_usage(lhs, &ext_func_lower, database);
        }

        let Some(func) = database.external_functions_get(&ext_func_lower) else {
            // trace!("TEMP_DEBUG: Unknown external function: {ext_func_lower}");
            return None;
        };
        let cmd_name = ext_func.as_str();
        let params = func.params();
        let ret = func
            .ret()
            .map(|r| GameValue::from_wiki_value(r, NilSource::FunctionReturn));
        if params.is_empty() {
            // no parameters to check
            return ret;
        }
        let Some(lhs) = lhs else {
            // Unary call, could retrive `_this` and check it as LHS, but for now it will just be `Anything`
            return ret;
        };
        // minimum required params (count from the end, first non-optional)
        let min_required_param = params.len()
            - params
                .iter()
                .rev()
                .position(|p| !p.optional())
                .unwrap_or(params.len());
        let expected_singular = if min_required_param <= 1 {
            // try matching raw first argument without array (`_unit call ace_common_fnc_isPlayer`)
            let arg_dummy = Arg::Item(String::from("0"));
            let (is_match, expected) =
                GameValue::match_set_to_arg(cmd_name, lhs, &arg_dummy, params);
            if is_match {
                return ret;
            }
            Some(expected)
        } else {
            None
        };
        let arg_dummy_vec = params
            .iter()
            .enumerate()
            .map(|(i, _p)| Arg::Item(format!("{i}")))
            .collect::<Vec<_>>();
        let arg_dummy = Arg::Array(arg_dummy_vec);
        let (is_match, mut expected) =
            GameValue::match_set_to_arg(cmd_name, lhs, &arg_dummy, params);
        if !is_match {
            if let Some(expected_singular) = expected_singular
                && !lhs.iter().all(|gv| matches!(gv, GameValue::Array(..)))
            {
                // if it could be singular and LHS may not be an array, report the singular expected type
                expected.extend(expected_singular);
            }
            self.errors.insert(Issue::InvalidArgs {
                command: ext_func_lower.clone(),
                span: span.clone(),
                variant: InvalidArgs::FuncTypeNotExpected {
                    expected: expected.into_iter().collect(),
                    found: lhs.iter().cloned().collect(),
                    span: span.clone(),
                },
            });
        }
        ret
    }

    /// Check usage of code blocks in external functions (e.g. `cba_fnc_execNextFrame`)
    fn external_check_code_usage(
        &mut self,
        lhs: &IndexSet<GameValue>,
        ext_func_lower: &str,
        database: &Database,
    ) {
        for possible in lhs {
            match possible {
                GameValue::Code(Some(statements)) => {
                    // handle `{} call cba_fnc_directcall`
                    if ext_func_lower == "cba_fnc_directcall" {
                        self.external_current_scope(
                            &vec![(GameValue::Code(Some(statements.clone())), statements.span())],
                            &vec![],
                            database,
                        );
                    }
                }
                GameValue::Array(Some(gv_array), _) => match ext_func_lower {
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
                                        ("_actionParams", GameValue::Anything),
                                    ],
                                    database,
                                );
                            }
                        }
                    }
                    "cba_fnc_addperframehandler"
                    | "cba_fnc_waitandexecute"
                    | "cba_fnc_execnextframe" => {
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
    pub fn external_new_scope(
        &mut self,
        code_arg: &Vec<(GameValue, Range<usize>)>,
        vars: &Vec<(&str, GameValue)>,
        database: &Database,
    ) {
        for (element, _) in code_arg {
            let GameValue::Code(Some(expression)) = element else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            self.scope_push(false, None);
            let stack_index = self.stack_push(Some(expression));
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
                self.eval_statements(statements, false, database);
                let _ = self.stack_pop(stack_index);
            }
            self.scope_pop();
        }
    }
    fn external_current_scope(
        &mut self,
        code_arg: &Vec<(GameValue, Range<usize>)>,
        vars: &Vec<(&str, GameValue)>,
        database: &Database,
    ) {
        for (element, _) in code_arg {
            let GameValue::Code(Some(expression)) = element else {
                continue;
            };
            let Expression::Code(statements) = expression else {
                continue;
            };
            let stack_index = self.stack_push(Some(expression));
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
            self.eval_statements(statements, true, database);
            self.stack_pop(stack_index);
        }
    }
}
