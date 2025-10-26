//! Emulates engine commands

use std::{ops::Range, vec};

use indexmap::IndexSet;

use crate::{
    Expression,
    analyze::inspector::{Issue, VarSource},
    parser::database::Database,
};

use super::{SciptScope, game_value::GameValue};

impl SciptScope {
    #[must_use]
    pub fn cmd_u_private(&mut self, rhs: &IndexSet<GameValue>) -> IndexSet<GameValue> {
        fn push_var(s: &mut SciptScope, var: &String, source: &Range<usize>) {
            if s.ignored_vars.contains(&var.to_ascii_lowercase()) {
                s.var_assign(
                    &var.to_string(),
                    true,
                    IndexSet::from([GameValue::Anything]),
                    VarSource::Ignore,
                );
            } else {
                s.var_assign(
                    &var.to_string(),
                    true,
                    IndexSet::from([GameValue::Nothing]),
                    VarSource::Private(source.clone()),
                );
            }
        }
        for possible in rhs {
            if let GameValue::Array(Some(gv_array), _) = possible {
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
        IndexSet::new()
    }
    #[must_use]
    pub fn cmd_generic_params(
        &mut self,
        rhs: &IndexSet<GameValue>,
        debug_type: &str,
        source: &Range<usize>,
    ) -> IndexSet<GameValue> {
        let mut error_type = String::new();
        for possible in rhs {
            let GameValue::Array(Some(gv_array), _) = possible else {
                continue;
            };

            for (gv_index_num, gv_index) in gv_array.iter().enumerate() {
                for element in gv_index {
                    match element {
                        GameValue::Anything | GameValue::Array(None, _) => {}
                        GameValue::String(_) => {
                            self.cmd_generic_params_element(
                                &[vec![element.clone()]], // put it in a dummy array
                                gv_index_num,
                                &mut error_type,
                            );
                        }
                        GameValue::Array(Some(arg_array), _) => {
                            self.cmd_generic_params_element(
                                arg_array,
                                gv_index_num,
                                &mut error_type,
                            );
                        }

                        _ => {
                            error_type = format!("{gv_index_num}: Element Type");
                        }
                    }
                }
            }
        }
        if !error_type.is_empty() {
            self.errors.insert(Issue::InvalidArgs(
                format!("{debug_type} - {error_type}"),
                source.clone(),
            ));
        }
        IndexSet::from([GameValue::Boolean(None)])
    }

    pub fn cmd_generic_params_element(
        &mut self,
        element: &[Vec<GameValue>],
        gv_index_num: usize,
        error_type: &mut String,
    ) {
        if element.is_empty() || element[0].is_empty() {
            return;
        }
        match &element[0][0] {
            GameValue::String(None) | GameValue::Anything => {}
            GameValue::String(Some(Expression::String(var_name, source, _))) => {
                if var_name.is_empty() {
                    return;
                }
                let mut var_types = IndexSet::new();
                if element.len() > 2 {
                    for type_p in &element[2] {
                        match type_p {
                            GameValue::Array(Some(type_array), _) => {
                                for type_i in type_array {
                                    var_types.extend(type_i.iter().map(GameValue::make_generic));
                                }
                            }
                            GameValue::Array(None, _) | GameValue::Anything => {}
                            _ => {
                                *error_type = format!("{gv_index_num}: Expected Data Types");
                            }
                        }
                    }
                }
                if var_types.is_empty() {
                    var_types.insert(GameValue::Anything);
                }
                // Add the default value to types
                // It would be nice to move this above the is_empty check but not always safe
                // ie: assume `params ["_z", ""]` is type string, but this is not guaranteed
                if element.len() > 1 && !element[1].is_empty() {
                    let default_value = element[1][0].clone();
                    // Verify that the default value matches one of the types
                    if !(matches!(default_value, GameValue::Anything)
                        || matches!(default_value, GameValue::Nothing)
                        || var_types.iter().any(|t| {
                            matches!(t, GameValue::Anything) || t == &default_value.make_generic()
                        }))
                    {
                        *error_type =
                            format!("{gv_index_num}: Default Value does not match declared types");
                    }
                    var_types.insert(default_value);
                }
                self.var_assign(
                    var_name.as_ref(),
                    true,
                    var_types,
                    VarSource::Params(source.clone()),
                );
            }
            _ => {
                *error_type = format!("{gv_index_num}: Element Type");
            }
        }
    }

    #[must_use]
    pub fn cmd_generic_call(
        &mut self,
        rhs: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
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
        IndexSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_b_do(
        &mut self,
        lhs: &IndexSet<GameValue>,
        rhs: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
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
                                    IndexSet::from([GameValue::Number(None)]),
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
        IndexSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_generic_call_magic(
        &mut self,
        code_possibilities: &IndexSet<GameValue>,
        magic: &Vec<(&str, GameValue)>,
        source: &Range<usize>,
        database: &Database,
    ) -> IndexSet<GameValue> {
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
                    IndexSet::from([value.clone()]),
                    VarSource::Magic(source.clone()),
                );
            }
            self.code_used.insert(expression.clone());
            self.eval_statements(statements, database);
            self.pop();
        }
        IndexSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_for(&mut self, rhs: &IndexSet<GameValue>) -> IndexSet<GameValue> {
        let mut return_value = IndexSet::new();
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
                GameValue::Array(option, _) => {
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
        lhs: &IndexSet<GameValue>,
        _rhs: &IndexSet<GameValue>,
    ) -> IndexSet<GameValue> {
        lhs.clone()
    }
    #[must_use]
    pub fn cmd_u_is_nil(
        &mut self,
        rhs: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
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
        IndexSet::from([GameValue::Boolean(None)])
    }
    #[must_use]
    pub fn cmd_b_then(
        &mut self,
        _lhs: &IndexSet<GameValue>,
        rhs: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut return_value = IndexSet::new();
        for possible in rhs {
            if let GameValue::Code(Some(Expression::Code(_statements))) = possible {
                return_value.extend(self.cmd_generic_call(rhs, database));
            }
            if let GameValue::Array(Some(gv_array), _) = possible {
                for gv_index in gv_array {
                    for element in gv_index {
                        if let GameValue::Code(Some(expression)) = element {
                            return_value.extend(self.cmd_generic_call(
                                &IndexSet::from([GameValue::Code(Some(expression.clone()))]),
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
        lhs: &IndexSet<GameValue>,
        rhs: &IndexSet<GameValue>,
    ) -> IndexSet<GameValue> {
        let mut return_value = IndexSet::new(); // just merge, not really the same but should be fine
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
        rhs: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut possible_code = IndexSet::new();
        for possible_outer in rhs {
            let GameValue::Array(Some(gv_array), _) = possible_outer else {
                continue;
            };
            if gv_array.len() < 2 {
                continue;
            }
            possible_code.extend(gv_array[1].clone());
        }
        let _ = self.cmd_generic_call(&possible_code, database);
        IndexSet::from([GameValue::Anything])
    }
    #[must_use]
    pub fn cmd_u_to_string(&mut self, rhs: &IndexSet<GameValue>) -> IndexSet<GameValue> {
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
        IndexSet::from([GameValue::String(None)])
    }
    #[must_use]
    pub fn cmd_b_select(
        &mut self,
        lhs: &IndexSet<GameValue>,
        rhs: &IndexSet<GameValue>,
        cmd_set: &IndexSet<GameValue>,
        source: &Range<usize>,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut return_value = cmd_set.clone();
        // Check: `array select expression`
        let _ =
            self.cmd_generic_call_magic(rhs, &vec![("_x", GameValue::Anything)], source, database);
        // if lhs is array, and rhs is bool/number then put array into return
        if lhs.len() == 1
            && rhs
                .iter()
                .any(|r| matches!(r, GameValue::Boolean(..)) || matches!(r, GameValue::Number(..)))
            && let Some(GameValue::Array(Some(gv_array), _)) = lhs.iter().next()
        {
            // return_value.clear(); // todo: could clear if we handle pushBack
            for gv_index in gv_array {
                for element in gv_index {
                    return_value.insert(element.clone());
                }
            }
        }
        return_value
    }

    pub fn cmd_eqx_count_lint(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
        database: &Database,
        equal_zero: bool,
    ) {
        let Expression::Number(float_ord::FloatOrd(0.0), _) = *rhs else {
            return;
        };
        let Expression::UnaryCommand(crate::UnaryCommand::Named(ref lhs_cmd), ref count_input, _) =
            *lhs
        else {
            return;
        };
        if lhs_cmd != "count" {
            return;
        }
        let count_input_set = self.eval_expression(count_input, database);
        if count_input_set.is_empty()
            || !count_input_set
                .iter()
                .all(|arr| matches!(arr, GameValue::Array(..)))
        {
            return;
        }
        self.errors.insert(Issue::CountArrayComparison(
            equal_zero,
            lhs.span().start..rhs.span().end,
            count_input.source(),
        ));
    }
    /// emulate a possibly modified l-value array by a command
    pub fn cmd_generic_modify_lvalue(&mut self, lhs: &Expression) {
        let Expression::Variable(var_name, _) = lhs else {
            return;
        };
        // if var currently contains a specialized array
        if !self
            .var_retrieve(var_name, &lhs.full_span(), true)
            .iter()
            .any(|v| matches!(v, GameValue::Array(Some(_), _)))
        {
            return;
        }
        // push a generic array
        self.var_assign(
            var_name,
            false,
            IndexSet::from([GameValue::Array(None, None)]),
            VarSource::Ignore,
        );
    }
}
