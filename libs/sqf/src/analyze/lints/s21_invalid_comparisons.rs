use std::{ops::Range, sync::Arc};

use float_ord::FloatOrd;
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity},
};

use crate::{analyze::SqfLintData, BinaryCommand, Expression, Statement, UnaryCommand};

crate::lint!(LintS21InvalidComparisons);

impl Lint<SqfLintData> for LintS21InvalidComparisons {
    fn ident(&self) -> &str {
        "invalid_comparisons"
    }

    fn sort(&self) -> u32 {
        210
    }

    fn description(&self) -> &str {
        "Checks for if statements with impossible or overlapping conditions"
    }

    fn documentation(&self) -> &str {
        r"### Example

**Incorrect**
```sqf
// This will never be true
if (_x < 20 && _x > 30) then { ... };
// If _x is less than 20, it will also be less than 10
if (_x < 20 && _x < 10) then { ... };
```

### Explanation

This lint checks for if statements with impossible or overlapping conditions. This can be caused by typos or incorrect logic. HEMTT is not able to determine the intent of the code, so it is up to the developer to fix the condition.
"
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
    type Target = crate::Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &SqfLintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::UnaryCommand(UnaryCommand::Named(name), arg, _) = target else {
            return Vec::new();
        };

        if name != "if" {
            return Vec::new();
        }

        let comparisions = extract_comparisons(arg);
        let flat = flatten_comparisons(comparisions);
        let issues = find_issues(flat);
        issues
            .into_iter()
            .map(|issue| {
                Arc::new(CodeS21InvalidComparisons::new(
                    issue,
                    processed,
                    config.severity(),
                )) as Arc<dyn Code>
            })
            .collect::<Vec<Arc<dyn Code>>>()
    }
}

#[derive(Debug)]
enum Comparison {
    LessThan(String, FloatOrd<f32>, Range<usize>),
    LessThanOrEqual(String, FloatOrd<f32>, Range<usize>),
    Equal(String, FloatOrd<f32>, Range<usize>),
    NotEqual(String, FloatOrd<f32>, Range<usize>),
    GreaterThanOrEqual(String, FloatOrd<f32>, Range<usize>),
    GreaterThan(String, FloatOrd<f32>, Range<usize>),
    CompareGroup(Vec<Comparison>),
    NonCompareGroup(Vec<Comparison>),
    Ignored,
}

impl Comparison {
    pub const fn var(&self) -> Option<&String> {
        match self {
            Self::LessThan(var, _, _)
            | Self::LessThanOrEqual(var, _, _)
            | Self::Equal(var, _, _)
            | Self::NotEqual(var, _, _)
            | Self::GreaterThanOrEqual(var, _, _)
            | Self::GreaterThan(var, _, _) => Some(var),
            _ => None,
        }
    }

    pub const fn span(&self) -> Option<&Range<usize>> {
        match self {
            Self::LessThan(_, _, span)
            | Self::LessThanOrEqual(_, _, span)
            | Self::Equal(_, _, span)
            | Self::NotEqual(_, _, span)
            | Self::GreaterThanOrEqual(_, _, span)
            | Self::GreaterThan(_, _, span) => Some(span),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ComparisonIssue {
    issue: ComparisonIssueType,
    span_a: Range<usize>,
    span_b: Range<usize>,
}

#[derive(Debug)]
pub enum ComparisonIssueType {
    Impossible,
    Overlapping,
}

/// Extracts comparisons that are grouped with &&
fn extract_comparisons(expr: &Expression) -> Comparison {
    match expr {
        Expression::BinaryCommand(cmd, lhs, rhs, _) => {
            let span = expr.full_span();
            match cmd {
                BinaryCommand::Or => {
                    let lhs = extract_comparisons(lhs);
                    let rhs = extract_comparisons(rhs);
                    Comparison::NonCompareGroup(vec![lhs, rhs])
                }
                BinaryCommand::And => {
                    let lhs = extract_comparisons(lhs);
                    let rhs = extract_comparisons(rhs);
                    Comparison::CompareGroup(vec![lhs, rhs])
                }
                BinaryCommand::Less => {
                    let Some((variable, number)) = extract_ident_number(lhs, rhs) else {
                        return Comparison::Ignored;
                    };
                    Comparison::LessThan(variable.clone(), *number, span)
                }
                BinaryCommand::LessEq => {
                    let Some((variable, number)) = extract_ident_number(lhs, rhs) else {
                        return Comparison::Ignored;
                    };
                    Comparison::LessThanOrEqual(variable.clone(), *number, span)
                }
                BinaryCommand::Eq => {
                    let Some((variable, number)) = extract_ident_number(lhs, rhs) else {
                        return Comparison::Ignored;
                    };
                    Comparison::Equal(variable.clone(), *number, span)
                }
                BinaryCommand::NotEq => {
                    let Some((variable, number)) = extract_ident_number(lhs, rhs) else {
                        return Comparison::Ignored;
                    };
                    Comparison::NotEqual(variable.clone(), *number, span)
                }
                BinaryCommand::GreaterEq => {
                    let Some((variable, number)) = extract_ident_number(lhs, rhs) else {
                        return Comparison::Ignored;
                    };
                    Comparison::GreaterThanOrEqual(variable.clone(), *number, span)
                }
                BinaryCommand::Greater => {
                    let Some((variable, number)) = extract_ident_number(lhs, rhs) else {
                        return Comparison::Ignored;
                    };
                    Comparison::GreaterThan(variable.clone(), *number, span)
                }
                _ => Comparison::Ignored,
            }
        }
        Expression::Code(code) => {
            if code.content().len() != 1 {
                return Comparison::Ignored;
            }
            if let Some(Statement::Expression(expr, _)) = code.content().first() {
                if matches!(expr, Expression::BinaryCommand(_, _, _, _)) {
                    extract_comparisons(expr)
                } else {
                    Comparison::Ignored
                }
            } else {
                Comparison::Ignored
            }
        }
        _ => Comparison::Ignored,
    }
}

const fn extract_ident_number<'a>(
    lhs: &'a Expression,
    rhs: &'a Expression,
) -> Option<(&'a String, &'a FloatOrd<f32>)> {
    let variable = {
        if let Expression::Variable(name, _) = lhs {
            name
        } else if let Expression::Variable(name, _) = rhs {
            name
        } else {
            return None;
        }
    };
    let number = {
        if let Expression::Number(num, _) = lhs {
            num
        } else if let Expression::Number(num, _) = rhs {
            num
        } else {
            return None;
        }
    };
    Some((variable, number))
}

/// Flatten comparisons that are grouped with && for easier comparison
fn flatten_comparisons(comparisons: Comparison) -> Comparison {
    match comparisons {
        Comparison::CompareGroup(comparisons) => {
            let mut flat = Vec::new();
            for comparison in comparisons {
                match comparison {
                    Comparison::CompareGroup(_) => {
                        let Comparison::CompareGroup(mut inner) = flatten_comparisons(comparison)
                        else {
                            panic!("Expected CompareGroup");
                        };
                        flat.append(&mut inner);
                    }
                    Comparison::NonCompareGroup(_) => {
                        let Comparison::NonCompareGroup(inner) =
                            flatten_comparisons(comparison)
                        else {
                            panic!("Expected NonCompareGroup");
                        };
                        flat.push(Comparison::NonCompareGroup(inner));
                    }
                    Comparison::Ignored => {}
                    _ => {
                        flat.push(comparison);
                    }
                }
            }
            Comparison::CompareGroup(flat)
        }
        Comparison::NonCompareGroup(comparisons) => {
            Comparison::NonCompareGroup(comparisons.into_iter().map(flatten_comparisons).collect())
        }
        _ => comparisons,
    }
}

fn find_issues(comparisons: Comparison) -> Vec<ComparisonIssue> {
    let mut issues = Vec::new();
    match comparisons {
        Comparison::CompareGroup(comparisons) => {
            for (i, a) in comparisons.iter().enumerate() {
                for b in comparisons.iter().skip(i + 1) {
                    if let Some(issue) = check_issue(a, b) {
                        issues.push(issue);
                    } else if let Some(issue) = check_issue(b, a) {
                        issues.push(issue);
                    }
                }
            }
        }
        Comparison::NonCompareGroup(comparisons) => {
            for comparison in comparisons {
                issues.append(&mut find_issues(comparison));
            }
        }
        _ => {}
    }
    issues
}

#[allow(clippy::too_many_lines)]
fn check_issue(a: &Comparison, b: &Comparison) -> Option<ComparisonIssue> {
    let a_var = a.var()?;
    let b_var = b.var()?;
    if a_var != b_var {
        return None;
    }
    let a_span = a.span()?.clone();
    let b_span = b.span()?.clone();
    match (a, b) {
        (
            Comparison::GreaterThan(_, _, _) | Comparison::GreaterThanOrEqual(_, _, _),
            Comparison::GreaterThan(_, _, _) | Comparison::GreaterThanOrEqual(_, _, _),
        )
        | (
            Comparison::LessThan(_, _, _) | Comparison::LessThanOrEqual(_, _, _),
            Comparison::LessThan(_, _, _) | Comparison::LessThanOrEqual(_, _, _),
        ) => Some(ComparisonIssue {
            issue: ComparisonIssueType::Overlapping,
            span_a: a_span,
            span_b: b_span,
        }),
        (
            Comparison::LessThan(_, max_a, _) | Comparison::LessThanOrEqual(_, max_a, _),
            Comparison::GreaterThan(_, min_b, _) | Comparison::GreaterThanOrEqual(_, min_b, _),
        ) => {
            if max_a < min_b {
                Some(ComparisonIssue {
                    issue: ComparisonIssueType::Impossible,
                    span_a: a_span,
                    span_b: b_span,
                })
            } else {
                None
            }
        }
        (Comparison::NotEqual(_, num_a, _), Comparison::NotEqual(_, num_b, _))
        | (Comparison::Equal(_, num_a, _), Comparison::Equal(_, num_b, _)) => {
            if num_a == num_b {
                Some(ComparisonIssue {
                    issue: ComparisonIssueType::Overlapping,
                    span_a: a_span,
                    span_b: b_span,
                })
            } else {
                Some(ComparisonIssue {
                    issue: ComparisonIssueType::Impossible,
                    span_a: a_span,
                    span_b: b_span,
                })
            }
        }
        (Comparison::NotEqual(_, num_a, _), Comparison::Equal(_, num_b, _))
        | (Comparison::Equal(_, num_a, _), Comparison::NotEqual(_, num_b, _)) => {
            if num_a == num_b {
                Some(ComparisonIssue {
                    issue: ComparisonIssueType::Impossible,
                    span_a: a_span,
                    span_b: b_span,
                })
            } else {
                Some(ComparisonIssue {
                    issue: ComparisonIssueType::Overlapping,
                    span_a: a_span,
                    span_b: b_span,
                })
            }
        }
        _ => None,
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS21InvalidComparisons {
    issue: ComparisonIssue,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS21InvalidComparisons {
    fn ident(&self) -> &'static str {
        "L-S21"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#invalid_comparisons")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        match self.issue.issue {
            ComparisonIssueType::Impossible => "Impossible comparison".to_string(),
            ComparisonIssueType::Overlapping => "Overlapping comparison".to_string(),
        }
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS21InvalidComparisons {
    #[must_use]
    pub fn new(issue: ComparisonIssue, processed: &Processed, severity: Severity) -> Self {
        Self {
            severity,
            diagnostic: None,
            issue,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) =
            Diagnostic::new_for_processed(&self, self.issue.span_a.clone(), processed)
        else {
            return self;
        };
        diag = diag.clear_labels();
        for span in &[self.issue.span_a.clone(), self.issue.span_b.clone()] {
            let Some(map_start) = processed.mapping(span.start) else {
                return self;
            };
            let Some(map_end) = processed.mapping(span.end) else {
                return self;
            };
            let Some(map_file) = processed.source(map_start.source()) else {
                return self;
            };
            diag.labels.push(Label::primary(
                map_file.0.clone(),
                map_start.original_start()..map_end.original_start(),
            ));
        }
        self.diagnostic = Some(diag);
        self
    }
}
