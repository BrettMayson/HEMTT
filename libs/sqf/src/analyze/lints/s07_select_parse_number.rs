use std::{ops::Range, sync::Arc};

use arma3_wiki::model::Value;
use float_ord::FloatOrd;
use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::LintData, parser::database::Database, BinaryCommand, Expression};

crate::analyze::lint!(LintS07SelectParseNumber);

impl Lint<LintData> for LintS07SelectParseNumber {
    fn ident(&self) -> &'static str {
        "select_parse_number"
    }

    fn sort(&self) -> u32 {
        70
    }

    fn description(&self) -> &'static str {
        "Checks for `select` commands that can be replaced with `parseNumber`"
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```sqf
private _isWater = [0, 1] select (surfaceIsWater getPos player);
```
**Correct**
```sqf
private _isWater = parseNumber (surfaceIsWater getPos player);
```
**Incorrect**
```sqf
private _isLand = [1, 0] select (surfaceIsWater getPos player);
```
**Correct**
```sqf
private _isLand = parseNumber !(surfaceIsWater getPos player);
```

### Explanation

Using `select` on an array with 0 and 1 can be replaced with `parseNumber` for better performance."
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
        _project: Option<&ProjectConfig>,
        _build_info: Option<&hemtt_common::config::BuildInfo>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let (_, database) = data;
        let Expression::BinaryCommand(BinaryCommand::Named(name), expression, condition, _) = target
        else {
            return Vec::new();
        };
        if name.to_lowercase() != "select" {
            return Vec::new();
        }
        let Expression::Array(args, _) = &**expression else {
            return Vec::new();
        };
        if args.len() != 2 {
            return Vec::new();
        }
        let Expression::Number(FloatOrd(mut lhs), _) = &args[0] else {
            return Vec::new();
        };
        let Expression::Number(FloatOrd(mut rhs), _) = &args[1] else {
            return Vec::new();
        };
        if !(match &**condition {
            Expression::Code(_)
            | Expression::Number(_, _)
            | Expression::Array(_, _)
            | Expression::ConsumeableArray(_, _)
            | Expression::Variable(_, _) => false,
            Expression::String(_, _, _) | Expression::Boolean(_, _) => true,
            Expression::NularCommand(cmd, _) => safe_command(cmd.as_str(), database),
            Expression::UnaryCommand(cmd, _, _) => safe_command(cmd.as_str(), database),
            Expression::BinaryCommand(cmd, _, _, _) => safe_command(cmd.as_str(), database),
        }) {
            return Vec::new();
        }
        let mut negate = false;
        if rhs.abs() < f32::EPSILON {
            negate = true;
            std::mem::swap(&mut lhs, &mut rhs);
        }
        if lhs.abs() > f32::EPSILON || (rhs - 1.0).abs() > f32::EPSILON {
            return Vec::new();
        }
        vec![Arc::new(CodeS07SelectParseNumber::new(
            expression.full_span(),
            (**condition).clone(),
            processed,
            negate,
            config.severity(),
        ))]
    }
}

fn safe_command(command: &str, database: &Database) -> bool {
    if let "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" = command {
        return true;
    }
    let Some(cmd) = database.wiki().commands().get(command) else {
        return false;
    };
    cmd.syntax().iter().all(|s| match &s.ret().0 {
        Value::Boolean | Value::String => true,
        Value::OneOf(rets) => rets
            .iter()
            .all(|(r, _)| matches!(r, Value::String | Value::Boolean)),
        _ => false,
    })
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS07SelectParseNumber {
    span: Range<usize>,
    expr: Expression,
    negate: bool,

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS07SelectParseNumber {
    fn ident(&self) -> &'static str {
        "L-S07"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#select_parse_number")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("using `select` where `parseNumber` is more appropriate")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "parseNumber {}",
            if matches!(
                self.expr,
                Expression::UnaryCommand(_, _, _) | Expression::BinaryCommand(_, _, _, _)
            ) || self.negate
            {
                let mut display_negate = true;
                let expr = if self.negate {
                    if let Expression::BinaryCommand(BinaryCommand::NotEq, a, b, c) = &self.expr {
                        display_negate = false;
                        Expression::BinaryCommand(
                            BinaryCommand::Eq,
                            a.clone(),
                            b.clone(),
                            c.clone(),
                        )
                        .source()
                    } else if let Expression::BinaryCommand(BinaryCommand::Eq, a, b, c) = &self.expr
                    {
                        display_negate = false;
                        Expression::BinaryCommand(
                            BinaryCommand::NotEq,
                            a.clone(),
                            b.clone(),
                            c.clone(),
                        )
                        .source()
                    } else {
                        self.expr.source()
                    }
                } else {
                    self.expr.source()
                };
                format!(
                    "{}({})",
                    if self.negate && display_negate {
                        "!"
                    } else {
                        ""
                    },
                    expr
                )
            } else {
                self.expr.source()
            }
        ))
    }

    fn note(&self) -> Option<String> {
        if let Expression::BinaryCommand(BinaryCommand::NotEq, _, _, _) = &self.expr {
            if self.negate {
                return Some("!= is now ==".to_string());
            }
        } else if let Expression::BinaryCommand(BinaryCommand::Eq, _, _, _) = &self.expr {
            if self.negate {
                return Some("== is now !=".to_string());
            }
        } else if self.negate {
            return Some("The condition is now negated with !".to_string());
        }
        None
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS07SelectParseNumber {
    #[must_use]
    pub fn new(span: Range<usize>, expr: Expression, processed: &Processed, negate: bool, severity: Severity) -> Self {
        Self {
            span,
            expr,
            negate,

            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::range_plus_one)]
    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic =
            Diagnostic::from_code_processed(&self, self.span.start..self.span.end + 1, processed);
        self
    }
}
