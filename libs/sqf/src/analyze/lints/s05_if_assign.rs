use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Diagnostic, Processed, Severity}};

use crate::{analyze::{extract_constant, SqfLintData}, BinaryCommand, Expression, UnaryCommand};

crate::lint!(LintS05IfAssign);

impl Lint<SqfLintData> for LintS05IfAssign {
    fn ident(&self) -> &str {
        "if_assign"
    }

    fn description(&self) -> &str {
        "If assign"
    }

    fn documentation(&self) -> &str {
        "The if statement is an assignment"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;

impl LintRunner<SqfLintData> for Runner {
    type Target = Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &SqfLintData,
    ) -> hemtt_workspace::reporting::Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        if let Expression::BinaryCommand(BinaryCommand::Named(name), if_cmd, code, _) = target {
            if name.to_lowercase() == "then" {
                let Expression::UnaryCommand(UnaryCommand::Named(_), condition, _) = &**if_cmd else {
                    return Vec::new();
                };
                if let Expression::BinaryCommand(BinaryCommand::Else, lhs_expr, rhs_expr, _) = &**code {
                    let lhs = extract_constant(lhs_expr);
                    let rhs = extract_constant(rhs_expr);
                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        // Skip if consts are used in a isNil check (e.g. [x, 5] select (isNil "x") will error in scheduled)
                        if let Expression::UnaryCommand(UnaryCommand::Named(name), _, _) = &**condition
                        {
                            if name.to_lowercase() == "isnil" {
                                return Vec::new();
                            }
                        }
                        return vec![Arc::new(CodeS05IfAssign::new(
                            if_cmd.span(),
                            (condition.source(), condition.full_span()),
                            (lhs, lhs_expr.span()),
                            (rhs, rhs_expr.span()),
                            processed,
                            config.severity(),
                        ))];
                    }
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

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            String::from("assignment to if can be replaced with parseNumber")
        } else {
            String::from("assignment to if can be replaced with select")
        }
    }

    fn label_message(&self) -> String {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            String::from("use parseNumber")
        } else {
            String::from("use select")
        }
    }

    fn suggestion(&self) -> Option<String> {
        if self.lhs.0 .0 == "1" && self.rhs.0 .0 == "0" {
            Some(format!("parseNumber {}", self.condition.0.as_str(),))
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
        let haystack = &processed.as_str()[self.rhs.1.end..];
        let end_position = self.rhs.1.end + haystack.find('}').unwrap_or(0) + 1;
        self.diagnostic =
            Diagnostic::new_for_processed(&self, self.if_cmd.start..end_position, processed);
        self
    }
}
