use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_error::{tokens::Token, Code};
use tracing::error;

use crate::{Defines, DefinitionLibrary};

/// Tried to use `#if` on an undefined macro
pub struct IfUndefined {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
    /// The defines at the point of the error
    pub(crate) defines: Defines,
}

impl Code for IfUndefined {
    fn ident(&self) -> &'static str {
        "PE8"
    }

    fn message(&self) -> String {
        "attempted to use `#if` on an undefined macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "attempted to use `#if` on an undefined macro `{}`",
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
        let span = self.token.source().start().0..self.token.source().end().0;
        let did_you_mean = self
            .defines
            .similar_values(self.token.symbol().output().trim());
        let mut report = Report::build(
            ReportKind::Error,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.source().path_or_builtin(), span.start..span.end))
                .with_color(a)
                .with_message("undefined macro"),
        );
        if !did_you_mean.is_empty() {
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
            (
                self.token.source().path_or_builtin(),
                Source::from(self.token.source().path().map_or_else(String::new, |path| {
                    path.read_to_string().unwrap_or_default()
                })),
            ),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }
}
