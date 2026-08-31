use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression, Statement};

crate::analyze::lint!(LintS26ShortCircuitBoolVar);

impl Lint<LintData> for LintS26ShortCircuitBoolVar {
    fn ident(&self) -> &'static str {
        "short_circuit_bool_var"
    }

    fn sort(&self) -> u32 {
        260
    }

    fn description(&self) -> &'static str {
        "Checks for inefficent short ciruit evaulation"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
if (_test1 && {_test2}) then { };
if (_test1 && {_a isEqualTo _b}) then { };
```
**Correct**
```sqf
if (_test1 && _test2) then { };
if (_test1 && _a isEqualTo _b) then { };
```

### Explanation

Short circuit evaluation is not free: the right hand side is a code block that has to be
created and called. When the right hand side is just a boolean variable, or a comparison
between simple values, evaluating it eagerly is cheaper than the short circuit that skips it.

Comparisons are only reported when both sides are simple values (a variable, number, string
or boolean). A comparison that calls a command is left alone, because the short circuit is
often guarding it:
```sqf
if (count _array > 0 && {_array select 0 isEqualTo "x"}) then { }; // not reported
```

False positives are possible if a variable could be undefined, e.g.:
```sqf
someLogic = !isNil "z";
someLogic && {z}
```
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(hemtt_common::config::LintEnabled::Pedantic)
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

/// Comparisons that are cheaper to evaluate eagerly than to short circuit around.
fn is_comparison(cmd: &BinaryCommand) -> bool {
    match cmd {
        BinaryCommand::Eq
        | BinaryCommand::NotEq
        | BinaryCommand::Greater
        | BinaryCommand::Less
        | BinaryCommand::GreaterEq
        | BinaryCommand::LessEq => true,
        BinaryCommand::Named(name) => [
            "isEqualTo",
            "isNotEqualTo",
            "isEqualRef",
            "isNotEqualRef",
            "isEqualType",
        ]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate)),
        _ => false,
    }
}

/// A value that can be evaluated without calling a command, so it cannot error or have side effects.
const fn is_simple_operand(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Variable(..)
            | Expression::Number(..)
            | Expression::String(..)
            | Expression::Boolean(..)
    )
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(cmd, left, right, _) = target else {
            return Vec::new();
        };
        if !(matches!(cmd, BinaryCommand::Or) || matches!(cmd, BinaryCommand::And)) {
            return Vec::new();
        }
        let Expression::Code(statements)= &**right else {
            return Vec::new();
        };
        if statements.content().len() != 1 { 
            return Vec::new()
        }
        let Statement::Expression(ref inner, ref range) = statements.content()[0] else {
            return Vec::new();
        };
        let note = match inner {
            Expression::Variable(bool_var_name, _) => {
                // `!isNil "z" && {z}` is guarding against z being undefined, the { } is load bearing
                if let Expression::UnaryCommand(not_cmd, not_rhs, _) = &**left
                    && not_cmd.as_str().eq_ignore_ascii_case("!")
                    && let Expression::UnaryCommand(isnil_cmd, isnil_rhs, _) = &**not_rhs
                    && isnil_cmd.as_str().eq_ignore_ascii_case("isNil")
                    && let Expression::String(isnil_input_str, _, _) = &**isnil_rhs
                    && isnil_input_str.eq_ignore_ascii_case(bool_var_name)
                {
                    return Vec::new();
                }
                "remove the { } and use the variable directly (if safe to do so)"
            }
            // a comparison of simple values cannot error, so the short circuit is not guarding it
            Expression::BinaryCommand(inner_cmd, inner_left, inner_right, _)
                if is_comparison(inner_cmd)
                    && is_simple_operand(inner_left)
                    && is_simple_operand(inner_right) =>
            {
                "remove the { } and use the comparison directly, it is cheaper than the short circuit"
            }
            _ => return Vec::new(),
        };
        vec![Arc::new(CodeS26ShortCircuitBoolVar::new(
            range.clone(),
            processed,
            config.severity(),
            note,
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS26ShortCircuitBoolVar {
    span: Range<usize>,
    severity: Severity,
    note: &'static str,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS26ShortCircuitBoolVar {
    fn ident(&self) -> &'static str {
        "L-S26"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#short_circuit_bool_var")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Inefficent short circuit evaulation".to_string()
    }

    fn label_message(&self) -> String {
        "unnecessary { }".to_string()
    }

    fn note(&self) -> Option<String> {
        Some(self.note.to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}
impl CodeS26ShortCircuitBoolVar {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        processed: &Processed,
        severity: Severity,
        note: &'static str,
    ) -> Self {
        Self {
            span,
            severity,
            note,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
