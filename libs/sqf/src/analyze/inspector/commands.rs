//! Emulates engine commands

use std::{ops::Range, vec};

use indexmap::IndexSet;

use crate::{
    Expression, Statement,
    analyze::inspector::{InvalidArgs, Issue, VarSource, game_value::NilSource},
    parser::database::Database,
};

use super::{Inspector, game_value::GameValue};

type MagicVars<'a> = (&'a [(&'a str, GameValue)], &'a Range<usize>);

impl Inspector {
    #[must_use]
    pub fn cmd_u_private(&mut self, rhs: &IndexSet<GameValue>) -> IndexSet<GameValue> {
        fn push_var(s: &mut Inspector, var: &str, source: &Range<usize>) {
            if s.ignored_vars.contains(&var.to_ascii_lowercase()) {
                s.var_assign(
                    var,
                    true,
                    IndexSet::from([GameValue::Anything]),
                    VarSource::Ignore,
                );
            } else {
                s.var_assign(
                    var,
                    true,
                    IndexSet::from([GameValue::Nothing(NilSource::PrivateArray)]),
                    VarSource::Private(source.clone()),
                );
            }
        }
        for possible in rhs {
            if let GameValue::Array(Some(gv_array), _) = possible {
                for gv_index in gv_array {
                    for element in gv_index {
                        let (GameValue::String(Some(Expression::String(var, source, _))), _) =
                            element
                        else {
                            continue;
                        };
                        if var.is_empty() {
                            continue;
                        }
                        push_var(self, var, source);
                    }
                }
            }
            if let GameValue::String(Some(Expression::String(var, source, _))) = possible {
                if var.is_empty() {
                    continue;
                }
                push_var(self, var, source);
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
        let mut error_type = None;
        for possible in rhs {
            let GameValue::Array(Some(gv_array), _) = possible else {
                continue;
            };

            for gv_index in gv_array {
                for (element, element_span) in gv_index {
                    match element {
                        GameValue::Anything | GameValue::Array(None, _) => {}
                        GameValue::String(_) => {
                            if let Some(error) = self.cmd_generic_params_element(
                                &[vec![(element.clone(), element_span.clone())]], // put it in a dummy array
                            ) {
                                error_type = Some(error);
                            }
                        }
                        GameValue::Array(Some(arg_array), _) => {
                            if let Some(error) = self.cmd_generic_params_element(arg_array) {
                                error_type = Some(error);
                            }
                        }

                        _ => {
                            error_type = Some(InvalidArgs::TypeNotExpected {
                                expected: vec![
                                    GameValue::String(None),
                                    GameValue::Array(None, None),
                                ],
                                found: vec![element.clone()],
                                span: element_span.clone(),
                            });
                        }
                    }
                }
            }
        }
        if let Some(error) = error_type {
            self.errors.insert(Issue::InvalidArgs {
                command: debug_type.to_string(),
                variant: error,
                span: source.clone(),
            });
        }
        IndexSet::from([GameValue::Boolean(None)])
    }

    pub fn cmd_generic_params_element(
        &mut self,
        element: &[Vec<(GameValue, Range<usize>)>],
    ) -> Option<InvalidArgs> {
        if element.is_empty() || element[0].is_empty() {
            return None;
        }
        let mut error_type = None;
        let (value, _) = &element[0][0];
        match value {
            GameValue::String(None) | GameValue::Anything => {}
            GameValue::String(Some(Expression::String(var_name, span, _))) => {
                if var_name.is_empty() {
                    return None;
                }
                let mut var_types = IndexSet::new();
                if element.len() > 2 {
                    for (type_p, type_p_span) in &element[2] {
                        match type_p {
                            GameValue::Array(Some(type_array), _) => {
                                for type_i in type_array {
                                    var_types.extend(type_i.iter().map(|(v, _)| v.make_generic()));
                                }
                            }
                            GameValue::Array(None, _) | GameValue::Anything => {}
                            _ => {
                                error_type = Some(InvalidArgs::TypeNotExpected {
                                    expected: vec![GameValue::Array(None, None)],
                                    found: vec![type_p.clone()],
                                    span: type_p_span.clone(),
                                });
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
                    let default_value = element[1][0].0.clone();
                    // Verify that the default value matches one of the types
                    if !(matches!(default_value, GameValue::Anything)
                        || matches!(default_value, GameValue::Nothing(NilSource::ExplicitNil))
                        || var_types.iter().any(|t| {
                            matches!(t, GameValue::Anything) || t == &default_value.make_generic()
                        }))
                    {
                        error_type = Some(InvalidArgs::DefaultDifferentType {
                            expected: var_types.iter().cloned().collect(),
                            found: vec![default_value.clone()],
                            span: element[1][0].1.clone(),
                            default: Some(
                                (element[2]
                                    .first()
                                    .map(|(_, s)| s.clone())
                                    .unwrap_or_default()
                                    .start)
                                    ..(element[2]
                                        .last()
                                        .map(|(_, s)| s.clone())
                                        .unwrap_or_default()
                                        .end),
                            ),
                        });
                    }
                    var_types.insert(default_value);
                }
                self.var_assign(
                    var_name.as_ref(),
                    true,
                    var_types,
                    VarSource::Params(span.clone()),
                );
            }
            _ => {
                error_type = Some(InvalidArgs::TypeNotExpected {
                    expected: vec![GameValue::String(None)],
                    found: vec![element[0][0].0.clone()],
                    span: element[0][0].1.clone(),
                });
            }
        }
        error_type
    }
    #[must_use]
    pub fn cmd_generic_call(
        &mut self,
        code_possibilities: &IndexSet<GameValue>,
        magic_opt: Option<MagicVars>,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut return_value = IndexSet::new();
        for possible in code_possibilities {
            let GameValue::Code(Some(expression)) = possible else {
                return_value.insert(GameValue::Anything);
                continue;
            };
            let Expression::Code(statements) = expression else {
                return_value.insert(GameValue::Anything);
                continue;
            };
            let stack_index = self.stack_push(Some(expression));
            if stack_index.is_none() {
                return_value.insert(GameValue::Anything);
                continue;
            }
            if let Some((vars, source)) = magic_opt {
                for (var, value) in vars {
                    self.var_assign(
                        var,
                        true,
                        IndexSet::from([value.clone()]),
                        VarSource::Magic(source.clone()),
                    );
                }
            }
            self.eval_statements(statements, true, database);
            return_value.extend(self.stack_pop(stack_index));
        }
        return_value
    }
    #[must_use]
    pub fn cmd_b_do(
        &mut self,
        lhs: &IndexSet<GameValue>,
        rhs: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut return_value = IndexSet::new();
        for possible in rhs {
            let GameValue::Code(Some(expression)) = possible else {
                return_value.insert(GameValue::Anything);
                continue;
            };
            let Expression::Code(statements) = expression else {
                return_value.insert(GameValue::Anything);
                continue;
            };
            let stack_index = self.stack_push(Some(expression));
            if stack_index.is_none() {
                return_value.insert(GameValue::Anything);
                continue;
            }
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
                                    IndexSet::from([GameValue::Number(None, None)]),
                                    VarSource::ForLoop(source.clone()),
                                );
                            }
                            Expression::Code(stage_statement) => {
                                self.code_used(stage);
                                self.eval_statements(stage_statement, false, database);
                            }
                            _ => {}
                        }
                    }
                } else {
                    do_run = true;
                }
            }
            if do_run {
                let add_final =
                    if let Some(Statement::Expression(Expression::UnaryCommand(named, _, _), _)) =
                        statements.content().last()
                    {
                        // if we know we end on a default, then we don't need to add it's nil return
                        !named.as_str().eq_ignore_ascii_case("default")
                    } else {
                        true
                    };
                self.eval_statements(statements, add_final, database);
            }
            return_value.extend(self.stack_pop(stack_index));
        }
        return_value
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
                        for (gv, _) in stage {
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
            let _ = self.cmd_generic_call(rhs, None, database);
        }
        IndexSet::from([GameValue::Boolean(None)])
    }
    #[must_use]
    pub fn cmd_b_then(
        &mut self,
        rhs: &Expression,
        rhs_set: &IndexSet<GameValue>,
        database: &Database,
    ) -> IndexSet<GameValue> {
        let mut return_value = IndexSet::new();
        for possible in rhs_set {
            if let GameValue::Code(Some(Expression::Code(_statements))) = possible {
                return_value.extend(self.cmd_generic_call(
                    &IndexSet::from([possible.clone()]),
                    None,
                    database,
                ));
            }
            if let GameValue::Array(Some(gv_array), _) = possible {
                for gv_index in gv_array {
                    for (element, _) in gv_index {
                        if let GameValue::Code(Some(expression)) = element {
                            return_value.extend(self.cmd_generic_call(
                                &IndexSet::from([GameValue::Code(Some(expression.clone()))]),
                                None,
                                database,
                            ));
                        }
                    }
                }
            }
        }
        // if without else branch, add a possible nil result in addition to then-branch results
        if let Expression::Code(_) = rhs {
            return_value.insert(GameValue::Nothing(NilSource::IfWithoutElse));
        }
        return_value
    }
    #[must_use]
    /// just merge both sides (this is equilivalent to generating an else array)
    pub fn cmd_b_else(
        &self,
        lhs: &IndexSet<GameValue>,
        rhs: &IndexSet<GameValue>,
    ) -> IndexSet<GameValue> {
        lhs.iter().chain(rhs.iter()).cloned().collect()
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
            possible_code.extend(gv_array[1].iter().map(|(gv, _)| gv.clone()));
        }
        let _ = self.cmd_generic_call(&possible_code, None, database);
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
            self.code_used(expression);
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
        let _ = self.cmd_generic_call(
            rhs,
            Some((&[("_x", GameValue::Anything)], source)),
            database,
        );
        // if lhs is array, and rhs is bool/number then put array into return
        if lhs.len() == 1
            && rhs
                .iter()
                .any(|r| matches!(r, GameValue::Boolean(..)) || matches!(r, GameValue::Number(..)))
            && let Some(GameValue::Array(Some(gv_array), _)) = lhs.iter().next()
        {
            // return_value.clear(); // todo: could clear if we handle pushBack
            for gv_index in gv_array {
                for (element, _) in gv_index {
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
                .all(|(arr, _)| matches!(arr, GameValue::Array(..)))
        {
            return;
        }
        self.errors.insert(Issue::CountArrayComparison(
            equal_zero,
            lhs.span().start..rhs.span().end,
            count_input.source(false),
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
            .any(|(v, _)| matches!(v, GameValue::Array(Some(_), _)))
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
