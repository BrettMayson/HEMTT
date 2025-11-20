use hemtt_common::config::LintConfig;
use hemtt_workspace::lint::{AnyLintRunner, Lint, LintRunner};

use crate::{analyze::LintData, Expression};

mod abs;
// mod atan2;
mod ceil;
mod clamp;
mod distance;
mod floor;
mod linear_conversion;
mod max;
mod min;
mod modulo;
mod pi;
crate::analyze::lint!(LintS33ReimplementingCommand);

/// Check if two expressions match, considering variables and optionally unwrapping code blocks
fn expressions_match(expr1: &Expression, expr2: &Expression, unwrap_code: bool) -> bool {
    let expr1 = if unwrap_code { unwrap_code_block(expr1) } else { expr1 };
    let expr2 = if unwrap_code { unwrap_code_block(expr2) } else { expr2 };
    
    match (expr1, expr2) {
        (Expression::Variable(v1, _), Expression::Variable(v2, _)) => v1 == v2,
        (Expression::Number(n1, _), Expression::Number(n2, _)) => (n1.0 - n2.0).abs() < f32::EPSILON,
        _ => false,
    }
}

/// Unwrap code blocks to get the inner expression
fn unwrap_code_block(expr: &Expression) -> &Expression {
    if let Expression::Code(statements) = expr
        && let Some(crate::Statement::Expression(inner, _)) = statements.content().first()
    {
        return inner;
    }
    expr
}

impl Lint<LintData> for LintS33ReimplementingCommand {
    fn ident(&self) -> &'static str {
        "reimplementing_command"
    }

    fn sort(&self) -> u32 {
        330
    }

    fn description(&self) -> &'static str {
        "Checks if a code could be replaced with a command"
    }

    fn documentation(&self) -> &'static str {
r"### Example

**Incorrect**
```sqf
private _newValue = 10 + ((25 - 0) * (100 - 10) / (0 - 0));
```
**Correct**
```sqf
private _newValue = linearConversion [0, 0, 25, 10, 100];
```

### Explanation

Some code patterns can be more efficiently implemented using built-in commands. This lint identifies such patterns and suggests using the appropriate command for better performance and readability."
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;

impl LintRunner<LintData> for Runner {
    type Target = Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> hemtt_workspace::reporting::Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let mut codes = Vec::new();
        // Check clamp first as it's a more complex pattern that contains min/max
        codes.extend(clamp::check(target, processed, config));
        if !codes.is_empty() {
            return codes;
        }
        codes.extend(abs::check(target, processed, config));
        // https://github.com/acemod/ACE3/pull/6773/files#r250479159
        // codes.extend(atan2::check(target, processed, config));
        codes.extend(ceil::check(target, processed, config));
        codes.extend(distance::check(target, processed, config));
        codes.extend(floor::check(target, processed, config));
        codes.extend(linear_conversion::check(target, processed, config));
        codes.extend(max::check(target, processed, config));
        codes.extend(min::check(target, processed, config));
        codes.extend(modulo::check(target, processed, config));
        codes.extend(pi::check(target, processed, config));
        codes
    }
}
