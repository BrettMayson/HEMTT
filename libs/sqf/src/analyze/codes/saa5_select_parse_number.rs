use std::ops::Range;

use hemtt_common::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression};

pub struct SelectParseNumber {
    span: Range<usize>,
    expr: Expression,
    negate: bool,

    diagnostic: Option<Diagnostic>,
}

impl Code for SelectParseNumber {
    fn ident(&self) -> &'static str {
        "SAA5"
    }

    fn severity(&self) -> Severity {
        Severity::Help
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

impl SelectParseNumber {
    #[must_use]
    pub fn new(span: Range<usize>, expr: Expression, processed: &Processed, negate: bool) -> Self {
        Self {
            span,
            expr,
            negate,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::range_plus_one)]
    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic =
            Diagnostic::new_for_processed(&self, self.span.start..self.span.end + 1, processed);
        self
    }
}
