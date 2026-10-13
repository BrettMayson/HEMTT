use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{
    BinaryCommand::{self},
    Expression, Statement,
    analyze::LintData,
};

crate::analyze::lint!(LintS56NotShortCircuiting);

impl Lint<LintData> for LintS56NotShortCircuiting {
    fn ident(&self) -> &'static str {
        "not_short_circuiting"
    }
    fn sort(&self) -> u32 {
        560
    }
    fn description(&self) -> &'static str {
        "Checks for logic chains that are not using short-circuiting properly"
    }
    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
a && {b} && {c}
```

**Correct**
```sqf
a && {b && {c}}
```

### Explanation

The first example only partialy short circuts.
`c` won't be evaulated if the previous are false, but the `&&` operator still has to run.
See https://community.bistudio.com/wiki/Code_Optimisation#Lazy_Evaluation


### Configuration

- **only_code**: Only checks for `(a || {b} || {c})` chains that are attempting to use short circuiting, false will check for any logic chains
- **threshold**: The minimum length of a logic chain to trigger the lint (min 2)

```toml
[lints.sqf.not_short_circuiting]
options.only_code = true
options.threshold = 5
```
"
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help()
        // LintConfig::help().with_enabled(hemtt_common::config::LintEnabled::Pedantic)
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}
struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Statements;

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
        // default of `true` will reduce false positives and only catch code that was attempting to short-circuit (and avoid conflict with s26)
        let only_code = config.option("only_code").and_then(toml::Value::as_bool).unwrap_or(true);
        // or/and ops are still very fast, so threshold will reduce unnecessary linting for short chains
        let threshold = usize::try_from(config.option("threshold").and_then(toml::Value::as_integer).unwrap_or(5)).expect("usize").max(2);
        let mut context = Context { codes: vec![], processed, config, threshold, only_code, };

        for statement in target.content() {
            let (Statement::Expression(expr, _) | Statement::AssignGlobal(_, expr, _) | Statement::AssignLocal(_, expr, _)) = statement;
            check_expression(expr, None, &mut context);
        }
        context.codes
    }
}

const COMMANDS: [BinaryCommand; 2] = [BinaryCommand::And, BinaryCommand::Or];
struct Context<'a> {
    codes: Codes,
    processed: &'a Processed,
    config: &'a LintConfig,
    threshold: usize,
    only_code: bool,
}

/// Recursively check an expression for logic chains (that are not using the parent's op)
fn check_expression(expr: &Expression, parent: Option<&BinaryCommand>, context: &mut Context) {
    match expr {
        Expression::UnaryCommand(_, rhs, _) => check_expression(rhs, None, context),
        Expression::BinaryCommand(bcmd, left, right, _) => {
            check_expression(left, Some(bcmd), context);
            check_expression(right, Some(bcmd), context);
            if COMMANDS.contains(bcmd) && parent.is_none_or(|p| p != bcmd) {
                let mut logic_chain: Vec<&Expression> = Vec::new();
                get_logic_chain(expr, bcmd, &mut logic_chain, context);
                let op_len = logic_chain.len().saturating_sub(1);
                if op_len >= context.threshold {
                    context.codes.push(Arc::new(CodeS56NotShortCircuiting::new(
                            expr.span(),
                            op_len,
                            bcmd.as_str().to_string(),
                            context.processed,
                            context.config.severity(),
                        )));
                }
            }
        }
        Expression::Array(arr, _) => {
            for element in arr {
                check_expression(element, None, context);
            }
        }
        _ => {}
    }
}
/// Recursively collects a chain of expressions connected by the same logic command.
fn get_logic_chain<'a>(
    expr: &'a Expression,
    command: &BinaryCommand,
    chain: &mut Vec<&'a Expression>,
    lint_info: &Context,
) {
    let Expression::BinaryCommand(expr_command, left, right, _) = expr else {
        chain.push(expr);
        return;
    };
    if expr_command != command {
        chain.push(expr);
        return;
    }
    if (lint_info.only_code) && !matches!(**right, Expression::Code(..)) {
        return;
    }
    get_logic_chain(left, command, chain, lint_info);
    get_logic_chain(right, command, chain, lint_info);
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS56NotShortCircuiting {
    span: Range<usize>,
    length: usize,
    command: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS56NotShortCircuiting {
    fn ident(&self) -> &'static str {
        "L-S56"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#not_short_circuiting")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn label_message(&self) -> String {
        "not short circuiting".to_string()
    }
    fn message(&self) -> String {
        format!(
            "Chain of {} `{}` operations are not efficiently short circuiting",
            self.length, self.command
        )
    }
    fn help(&self) -> Option<String> {
        Some("Consider nesting the conditions to achieve short-circuiting".to_string())
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS56NotShortCircuiting {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        length: usize,
        command: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            command,
            length,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }
    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
