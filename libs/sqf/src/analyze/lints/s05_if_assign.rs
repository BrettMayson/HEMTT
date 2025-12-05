use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Diagnostic, Processed, Severity}};

use crate::{analyze::{extract_constant, LintData}, BinaryCommand, Expression, UnaryCommand};

crate::analyze::lint!(LintS05IfAssign);

impl Lint<LintData> for LintS05IfAssign {
    fn ident(&self) -> &'static str {
        "if_assign"
    }

    fn sort(&self) -> u32 {
        50
    }

    fn description(&self) -> &'static str {
        "Checks if statements that are used as assignments when select or parseNumber would be more appropriate"
    }

    fn documentation(&self) -> &'static str {
r#"### Example

**Incorrect**
```sqf
private _x = if (_myVar) then {1} else {0};
```
**Correct**
```sqf
private _x = parseNumber _myVar;
```
**Incorrect**
```sqf
private _x = if (_myVar) then {"apple"} else {"orange"};
```
**Correct**
```sqf
private _x = ["orange", "apple"] select _myVar;
```

### Explanation

`if` statements that are used as assignments and only return a static value can be replaced with the faster `select` or `parseNumber` commands."#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
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
        if let Expression::BinaryCommand(BinaryCommand::Named(name), if_cmd, code, _) = target
            && name.eq_ignore_ascii_case("then") {
                let Expression::UnaryCommand(UnaryCommand::Named(_), condition, _) = &**if_cmd else {
                    return Vec::new();
                };
                if let Expression::BinaryCommand(BinaryCommand::Else, lhs_expr, rhs_expr, _) = &**code {
                    let lhs = extract_constant(lhs_expr);
                    let rhs = extract_constant(rhs_expr);
                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        // Skip if consts are used in a isNil check (e.g. [x, 5] select (isNil "x") will error in scheduled)
                        if let Expression::UnaryCommand(UnaryCommand::Named(name), _, _) = &**condition
                            && name.eq_ignore_ascii_case("isnil") {
                                return Vec::new();
                            }
                        return vec![Arc::new(CodeS05IfAssign::new(
                            if_cmd.span(),
                            (condition.source(false), condition.full_span()),
                            (lhs, lhs_expr.span()),
                            (rhs, rhs_expr.span()),
                            processed,
                            config.severity(),
                        ))];
                    }
                }
            }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS05IfAssign {
    if_cmd: Range<usize>,
    condition: (String, Range<usize>),
    lhs: ((String, bool), Range<usize>),
    rhs: ((String, bool), Range<usize>),

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS05IfAssign {
    fn ident(&self) -> &'static str {
        "L-S05"
    }
    
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#if_assign")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            String::from("assignment to if can be replaced with parseNumber")
        } else if self.lhs.0 == (String::from("true"), false) && self.rhs.0 == (String::from("false"), false) {
            String::from("assignment to if can be replaced the condition directly")
        } else if self.lhs.0 == (String::from("false"), false) && self.rhs.0 == (String::from("true"), false) {
            String::from("assignment to if can be replaced the inverted condition")
        } else {
            String::from("assignment to if can be replaced with select")
        }
    }

    fn label_message(&self) -> String {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            String::from("use parseNumber")
        } else if self.lhs.0 == (String::from("true"), false) && self.rhs.0 == (String::from("false"), false) {
            String::from("use condition directly")
        } else if self.lhs.0 == (String::from("false"), false) && self.rhs.0 == (String::from("true"), false) {
            String::from("use inverted condition directly")
        } else {
            String::from("use select")
        }
    }

    fn suggestion(&self) -> Option<String> {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            Some(format!("parseNumber {}", self.condition.0.as_str(),))
        } else if self.lhs.0 == (String::from("true"), false) && self.rhs.0 == (String::from("false"), false) {
            Some(self.condition.0.as_str().to_string())
        } else if self.lhs.0 == (String::from("false"), false) && self.rhs.0 == (String::from("true"), false) {
            Some(format!("!({})", self.condition.0.as_str()))
        } else {
            Some(format!(
                "[{}, {}] select ({})",
                if self.rhs.0 .1 {
                    format!("\"{}\"", self.rhs.0 .0.as_str())
                } else {
                    self.rhs.0 .0.clone()
                },
                if self.lhs.0 .1 {
                    format!("\"{}\"", self.lhs.0 .0.as_str())
                } else {
                    self.lhs.0 .0.clone()
                },
                self.condition.0.as_str(),
            ))
        }
    }

    fn note(&self) -> Option<String> {
        Some(
            if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
                "parseNumber returns 1 for true and 0 for false"
            } else if self.lhs.0 == (String::from("true"), false) && self.rhs.0 == (String::from("false"), false) {
                "the if statement returns boolean values directly"
            } else if self.lhs.0 == (String::from("false"), false) && self.rhs.0 == (String::from("true"), false) {
                "the if statement returns inverted boolean value directly"
            } else {
                "the if and else blocks only return constant values\nselect is faster in this case"
            }
            .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS05IfAssign {
    #[must_use]
    pub fn new(
        if_cmd: Range<usize>,
        condition: (String, Range<usize>),
        lhs: ((String, bool), Range<usize>),
        rhs: ((String, bool), Range<usize>),
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            if_cmd,
            condition,
            lhs,
            rhs,

            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let haystack = &processed.extract(self.rhs.1.end..processed.output_chars());
        let end_position = self.rhs.1.end + haystack.find('}').unwrap_or(0) + 1;
        self.diagnostic =
            Diagnostic::from_code_processed(&self, self.if_cmd.start..end_position, processed);
        self
    }
}
