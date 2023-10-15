use std::rc::Rc;

use ariadne::{sources, ColorGenerator, Label, Report, ReportKind};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// Unexpected token
pub struct IfIncompatibleType {
    /// Left side of the operator
    pub(crate) left: (Vec<Token>, bool),
    /// Operator
    pub(crate) operator: Vec<Token>,
    /// Right side of the operator
    pub(crate) right: (Vec<Token>, bool),
}

impl IfIncompatibleType {
    pub fn new(
        left: (Vec<Rc<Token>>, bool),
        operator: Vec<Rc<Token>>,
        right: (Vec<Rc<Token>>, bool),
    ) -> Self {
        Self {
            left: (
                left.0.into_iter().map(|t| t.as_ref().clone()).collect(),
                left.1,
            ),
            operator: operator.into_iter().map(|t| t.as_ref().clone()).collect(),
            right: (
                right.0.into_iter().map(|t| t.as_ref().clone()).collect(),
                right.1,
            ),
        }
    }
}

impl Code for IfIncompatibleType {
    fn ident(&self) -> &'static str {
        "PE15"
    }

    fn message(&self) -> String {
        "incompatible types for operator in #if".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "incompatible types for operator in #if `{}` & `{}`",
            self.left
                .0
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<String>(),
            self.right
                .0
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<String>()
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.operator.first().unwrap().position().start().0
            ..self.operator.last().unwrap().position().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.left.0.first().unwrap().position().path().to_string(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_labels({
            let mut labels = vec![Label::new((
                self.operator.first().unwrap().position().path().to_string(),
                span.start..span.end,
            ))
            .with_color(colors.next())
            .with_message("operator only supports numbers")];
            if self.left.1 {
                labels.push(
                    Label::new((
                        self.left.0.first().unwrap().position().path().to_string(),
                        self.left.0.first().unwrap().position().start().0
                            ..self.left.0.last().unwrap().position().end().0,
                    ))
                    .with_color(colors.next())
                    .with_message("left side of operator"),
                );
            }
            if self.right.1 {
                labels.push(
                    Label::new((
                        self.right.0.first().unwrap().position().path().to_string(),
                        self.right.0.first().unwrap().position().start().0
                            ..self.right.0.last().unwrap().position().end().0,
                    ))
                    .with_color(colors.next())
                    .with_message("right side of operator"),
                );
            }
            labels.into_iter()
        })
        .finish()
        .write_for_stdout(
            sources(vec![
                (
                    self.operator.first().unwrap().position().path().to_string(),
                    self.operator
                        .first()
                        .unwrap()
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
                (
                    self.left.0.first().unwrap().position().path().to_string(),
                    self.left
                        .0
                        .first()
                        .unwrap()
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
                (
                    self.right.0.first().unwrap().position().path().to_string(),
                    self.right
                        .0
                        .first()
                        .unwrap()
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
            ]),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    #[cfg(feature = "lsp")]
    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.left.0.first().unwrap().position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.left.0.first().unwrap().position().start().to_lsp(),
                end: self.right.0.last().unwrap().position().end().to_lsp(),
            }),
        ))
    }
}
