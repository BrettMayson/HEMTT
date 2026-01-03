use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// Unexpected token
pub struct IfIncompatibleType {
    /// Left side of the operator
    left: (Vec<Token>, bool),
    /// Operator
    operator: Vec<Token>,
    /// Right side of the operator
    right: (Vec<Token>, bool),
}

impl Code for IfIncompatibleType {
    fn ident(&self) -> &'static str {
        "PE15"
    }

    fn message(&self) -> String {
        "incompatible types for operator in #if".to_string()
    }

    fn label_message(&self) -> String {
        "incompatible types".to_string()
    }

    fn help(&self) -> Option<String> {
        let operator = self
            .operator
            .iter()
            .map(|t| t.symbol().to_string())
            .collect::<String>();
        if let "<" | ">" | "<=" | ">=" = operator.as_str() {
            Some(format!("only numbers are supported for {operator}"))
        } else {
            None
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let start = self.operator.first()?;
        let end = self.operator.last()?;
        let mut diag = Diagnostic::new(self.ident(), self.message()).with_label(
            Label::primary(
                start.position().path().clone(),
                start.position().span().start..end.position().span().end,
            )
            .with_message(self.label_message()),
        );
        if self.left.1 {
            let left_first = self
                .left
                .0
                .first()
                .expect("left side should have a first token");
            let left_last = self
                .left
                .0
                .last()
                .expect("left side should have a last token");
            diag = diag.with_label(
                Label::secondary(
                    left_first.position().path().clone(),
                    left_first.position().span().start..left_last.position().span().end,
                )
                .with_message(
                    if left_first.symbol().is_double_quote() {
                        "lhs, string is not supported"
                    } else {
                        "lhs"
                    }
                    .to_string(),
                ),
            );
        }
        if self.right.1 {
            let right_first = self
                .right
                .0
                .first()
                .expect("right side should have a first token");
            let right_last = self
                .right
                .0
                .last()
                .expect("right side should have a last token");
            diag = diag.with_label(
                Label::secondary(
                    right_first.position().path().clone(),
                    right_first.position().span().start..right_last.position().span().end,
                )
                .with_message(
                    if right_first.symbol().is_double_quote() {
                        "rhs, string is not supported"
                    } else {
                        "rhs"
                    }
                    .to_string(),
                ),
            );
        }
        if let Some(help) = self.help() {
            diag = diag.with_help(help);
        }
        Some(diag)
    }
}

impl IfIncompatibleType {
    #[must_use]
    pub fn new(
        left: &(Arc<Vec<Arc<Token>>>, bool),
        operator: Vec<Arc<Token>>,
        right: &(Arc<Vec<Arc<Token>>>, bool),
    ) -> Self {
        Self {
            left: (
                left.0.iter().map(|t| t.as_ref().clone()).collect(),
                left.1,
            ),
            operator: operator.into_iter().map(|t| t.as_ref().clone()).collect(),
            right: (
                right.0.iter().map(|t| t.as_ref().clone()).collect(),
                right.1,
            ),
        }
    }

    #[must_use]
    pub fn code(
        left: &(Arc<Vec<Arc<Token>>>, bool),
        operator: Vec<Arc<Token>>,
        right: &(Arc<Vec<Arc<Token>>>, bool),
    ) -> Error {
        Error::Code(Arc::new(Self::new(left, operator, right)))
    }
}
