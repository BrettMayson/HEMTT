use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, Statement, Statements, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS42ForRange);

impl Lint<LintData> for LintS42ForRange {
    fn ident(&self) -> &'static str {
        "for_range"
    }

    fn sort(&self) -> u32 {
        420
    }

    fn description(&self) -> &'static str {
        "Checks for `for [{},{},{}]` loops, where `for .. from .. to .. do` could be used instead"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
for [{_i = 0}, {_i < 10}, {_i = _i + 1}] do {
    // code
};
```
**Correct**
```sqf
for _i from 0 to 10 do {
    // code
};
```

### Explanation

For loops using `for [{},{},{}]` are less efficient than using `for .. from .. to .. do`. The latter is more readable and performs better, as it avoids the overhead of evaluating the loop condition and increment expressions on each iteration.
"
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
        let Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, _, _) = target else {
            return Vec::new();
        };
        
        if !cmd.eq_ignore_ascii_case("do") {
            return Vec::new();
        }

        let Expression::UnaryCommand(UnaryCommand::Named(unary_cmd), lhs, for_span) = lhs.as_ref() else {
            return Vec::new();
        };
        if !unary_cmd.eq_ignore_ascii_case("for") {
            return Vec::new();
        }
        let span_start = for_span.start;

        let Expression::Array(parts, _) = lhs.as_ref() else {
            return Vec::new();
        };

        // First part is AssignGlobal
        let Some(Expression::Code(Statements { content: statements, .. })) = parts.first() else {
            return Vec::new();
        };
        if statements.len() != 1 {
            return Vec::new();
        }
        let Some(Statement::AssignGlobal(variable, from_value, _)) = statements.first() else {
            return Vec::new();
        };
        #[allow(clippy::cast_possible_truncation)]
        let from_value = match from_value {
            Expression::Number(n, _) => n.0 as i32,
            _ => return Vec::new(),
        };

        // Second part is a comparison
        let Some(Expression::Code(Statements { content: statements, .. })) = parts.get(1) else {
            return Vec::new();
        };
        if statements.len() != 1 {
            return Vec::new();
        }
        let Some(Statement::Expression(Expression::BinaryCommand(cmp_cmd, _, rhs, _), _)) = statements.first() else {
            return Vec::new();
        };
        match cmp_cmd {
            BinaryCommand::Less | BinaryCommand::LessEq | BinaryCommand::Greater | BinaryCommand::GreaterEq => {},
            _ => return Vec::new(),
        }
        #[allow(clippy::cast_possible_truncation)]
        let to_value = match rhs.as_ref() {
            Expression::Number(n, _) => n.0 as i32,
            _ => return Vec::new(),
        };

        // Third part is an increment
        let Some(Expression::Code(Statements { content: statements, .. })) = parts.get(2) else {
            return Vec::new();
        };
        if statements.len() != 1 {
            return Vec::new();
        }
        let Some(Statement::AssignGlobal(_, assignment, _)) = statements.first() else {
            return Vec::new();
        };
        let Expression::BinaryCommand(inc_command, _, inc_rhs, _) = assignment else {
            return Vec::new();
        };
        #[allow(clippy::cast_possible_truncation)]
        let step = match inc_command {
            BinaryCommand::Add => match **inc_rhs {
                Expression::Number(n, _) => n.0 as i32,
                _ => return Vec::new(),
            },
            BinaryCommand::Sub => match **inc_rhs {
                Expression::Number(n, _) => -(n.0 as i32),
                _ => return Vec::new(),
            },
            _ => return Vec::new(),
        };

        vec![
            Arc::new(CodeS42ForRange::new(
                span_start..target.span().end,
                processed,
                config.severity(),
                variable.clone(),
                from_value,
                to_value,
                step,
            ))
        ]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS42ForRange {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
    variable: String,
    from_value: i32,
    to_value: i32,
    step: i32,
}

impl Code for CodeS42ForRange {
    fn ident(&self) -> &'static str {
        "L-S42"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#for_range")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("Use `for .. from .. to .. do` instead of `for [{{}},{{}},{{}}]`")
    }

    fn label_message(&self) -> String {
        "use `for .. from .. to .. do`".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        if self.step == 1 {
            Some(format!("for {} from {} to {} do", self.variable, self.from_value, self.to_value))
        } else {
            Some(format!("for {} from {} to {} step {} do", self.variable, self.from_value, self.to_value, self.step))
        }
    }

    // fn help(&self) -> Option<String> {
    //     Some("Remove `_this` from the call".to_string())
    // }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS42ForRange {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, variable: String, from_value: i32, to_value: i32, step: i32) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            variable,
            from_value,
            to_value,
            step,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
