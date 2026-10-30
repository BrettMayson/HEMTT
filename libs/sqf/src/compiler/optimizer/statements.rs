use std::sync::OnceLock;

use crate::{Expression, Statement, Statements};
#[allow(unused_imports)]
use tracing::{trace, warn};

static CALL_COMMANDS: OnceLock<Vec<String>> = OnceLock::new();

#[must_use]
fn code_call_commands() -> &'static [String] {
    CALL_COMMANDS.get_or_init(|| {
        let mut commands = vec![
            "configclasses".to_lowercase(), // these take strings which are evaluated as code
            "configproperties".to_lowercase(),
            "isnil".to_lowercase(), // these are missing wiki params info
            "then".to_lowercase(),
            "foreach".to_lowercase(),
            "foreachreversed".to_lowercase(),
            "switch".to_lowercase(),
        ];
        let database = crate::parser::database::Database::a3(false); // todo: pass this in
        for (name, command) in database.wiki().commands().iter() {
            if command.syntax().iter().any(|s| {
                s.params()
                    .iter()
                    .any(|p| *p.typ() == arma3_wiki::model::Value::Code)
            }) {
                commands.push(name.to_lowercase());
            }
        }
        commands
    })
}

impl Statements {
    #[must_use]
    pub fn reduce_vars(mut self) -> Self {
        #[must_use]
        fn is_target_var(expression: &Expression, t_var: &str) -> bool {
            matches!(expression, Expression::Variable(e_var, _) if e_var.eq_ignore_ascii_case(t_var))
        }
        #[must_use]
        fn get_replacment_statment(original: &Statement, new_expression: Expression) -> Statement {
            match original {
                Statement::AssignGlobal(var, _, range) => {
                    Statement::AssignGlobal(var.clone(), new_expression, range.clone())
                }
                Statement::AssignLocal(var, _, range) => {
                    Statement::AssignLocal(var.clone(), new_expression, range.clone())
                }
                Statement::Expression(_, range) => {
                    Statement::Expression(new_expression, range.clone())
                }
            }
        }
        #[must_use]
        fn check_expression(expression: &Expression, vars_used: &mut Vec<String>) -> bool {
            #[must_use]
            fn is_code_call_command(command: &str) -> bool {
                code_call_commands()
                    .iter()
                    .any(|cmd| cmd.eq_ignore_ascii_case(command))
            }
            match expression {
                Expression::Code(..)
                | Expression::Boolean(..)
                | Expression::Number(..)
                | Expression::String(..) => true,
                Expression::Variable(var, _) => {
                    vars_used.push(var.to_lowercase());
                    true
                }
                Expression::NularCommand(n_cmd, _) => !is_code_call_command(n_cmd.as_str()),
                Expression::UnaryCommand(u_cmd, rhs, _) => {
                    check_expression(rhs, vars_used) && !is_code_call_command(u_cmd.as_str())
                }
                Expression::BinaryCommand(b_cmd, lhs, rhs, _) => {
                    check_expression(lhs, vars_used)
                        && check_expression(rhs, vars_used)
                        && !is_code_call_command(b_cmd.as_str())
                }
                Expression::Array(vec, _) | Expression::ConsumeableArray(vec, _) => {
                    vec.iter().all(|e| check_expression(e, vars_used))
                }
            }
        }

        let mut index = self.content.len().saturating_sub(1);
        let mut vars_used: Vec<String> = Vec::new();

        // reverse order, stopping before index 0 because we access [index -1]
        while index > 0 {
            let this_statement = &self.content[index];
            let (Statement::AssignGlobal(_, cur_exp, _)
            | Statement::AssignLocal(_, cur_exp, _)
            | Statement::Expression(cur_exp, _)) = this_statement;

            if !check_expression(cur_exp, &mut vars_used) {
                println!(
                    "{index}: stopping optimization because expression {:?} is not safe",
                    cur_exp.command_name()
                );
                break;
            }

            // See if statement above this one is a `private _var` assignment
            let Statement::AssignLocal(var, above_exp, above_range) = &self.content[index - 1]
            else {
                index -= 1;
                continue;
            };
            let target_var = var.to_lowercase();

            if vars_used.iter().filter(|v| *v == &target_var).count() != 1 {
                println!("{index}: skipping because {var} is used more than once bellow");
                index -= 1;
                continue;
            }

            let replacement = match cur_exp {
                Expression::Variable(_, _) => {
                    if is_target_var(cur_exp, &target_var) {
                        Some(Statement::Expression(
                            above_exp.clone(),
                            above_range.clone(),
                        ))
                    } else {
                        None
                    }
                }
                Expression::UnaryCommand(u_cmd, rhs, source) => {
                    if is_target_var(rhs, &target_var) {
                        Some(get_replacment_statment(
                            this_statement,
                            Expression::UnaryCommand(
                                u_cmd.clone(),
                                Box::new(above_exp.clone()),
                                source.clone(),
                            ),
                        ))
                    } else {
                        None
                    }
                }
                Expression::BinaryCommand(b_cmd, lhs, rhs, source) => {
                    if is_target_var(lhs, &target_var) {
                        Some(get_replacment_statment(
                            this_statement,
                            Expression::BinaryCommand(
                                b_cmd.clone(),
                                Box::new(above_exp.clone()),
                                rhs.clone(),
                                source.clone(),
                            ),
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(replacement) = replacement {
                println!(
                    "{index}: optimizing {:?} to bypass {target_var}",
                    cur_exp.command_name()
                );
                trace!(
                    "{index}: optimizing {:?} to bypass {target_var}",
                    cur_exp.command_name()
                );
                self.content.remove(index);
                self.content[index - 1] = replacement;
            }
            index -= 1;
        }
        self
    }
}
