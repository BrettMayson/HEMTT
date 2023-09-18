use ariadne::{sources, ColorGenerator, Fmt, Label, Report, ReportKind};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

use crate::defines::Defines;

#[allow(unused)]
/// Tried to use `#if` on a [`Unit`](crate::context::Definition::Unit) or [`FunctionDefinition`](crate::context::Definition::Function)
pub struct IfUnitOrFunction {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The defines at the point of the error
    pub(crate) defines: Defines,
}

impl Code for IfUnitOrFunction {
    fn ident(&self) -> &'static str {
        "PE7"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "attempted to use `#if` on a unit or function macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "attempted to use `#if` on a unit or function macro `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().start().0..self.token.position().end().0;
        let did_you_mean = self
            .defines
            .similar_values(self.token.symbol().output().trim());
        let defined = self
            .defines
            .get_readonly(&self.token.symbol().to_string())
            .unwrap();
        let mut report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.token.position().path().to_string(),
                span.start..span.end,
            ))
            .with_color(a)
            .with_message(if defined.1.is_unit() {
                "trying to use a unit macro in an `#if`"
            } else {
                "trying to use a function macro in an `#if`"
            }),
        )
        .with_label(
            Label::new((
                defined.0.position().path().to_string(),
                defined.0.position().start().0..defined.0.position().end().0,
            ))
            .with_color(a)
            .with_message(format!(
                "defined as a {} here",
                if defined.1.is_unit() {
                    "unit"
                } else {
                    "function"
                }
            )),
        );
        if did_you_mean.is_empty() {
            report = report.with_help(format!(
                "did you mean `#ifdef {}`",
                self.token.symbol().output().fg(a)
            ));
        } else {
            report = report.with_help(format!(
                "did you mean `{}`",
                did_you_mean
                    .into_iter()
                    .map(|dym| format!("{}", dym.fg(a)))
                    .collect::<Vec<_>>()
                    .join("`, `")
            ));
        }
        if let Err(e) = report.finish().write_for_stdout(
            sources(vec![
                (
                    self.token.position().path().to_string(),
                    self.token
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
                (
                    defined.0.position().path().to_string(),
                    defined
                        .0
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
        let Some(path) = self.token.position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.token.position().start().to_lsp(),
                end: self.token.position().end().to_lsp(),
            }),
        ))
    }
}
