use ariadne::{sources, ColorGenerator, Fmt, Label, Report, ReportKind};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

use crate::defines::Defines;

#[allow(unused)]
/// Tried to call a [`FunctionDefinition`](crate::context::FunctionDefinition) with the wrong number of arguments
pub struct FunctionCallArgumentCount {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The number of arguments that were expected
    pub(crate) expected: usize,
    /// The number of arguments that were found
    pub(crate) got: usize,
    /// The defines at the point of the error
    pub(crate) defines: Defines,
}

impl Code for FunctionCallArgumentCount {
    fn ident(&self) -> &'static str {
        "PE9"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!(
            "function call with incorrect number of arguments, expected `{}` got `{}`",
            self.expected, self.got
        )
    }

    fn label_message(&self) -> String {
        format!(
            "incorrect argument count, expected `{}` got `{}`",
            self.expected, self.got,
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.token.position().start().0..self.token.position().end().0;
        let a = colors.next();
        let defined = self
            .defines
            .get_readonly(self.token.symbol().to_string().trim())
            .unwrap();
        let func = defined.1.as_function().unwrap();
        let did_you_mean = self
            .defines
            .similar_function(self.token.symbol().to_string().trim(), Some(self.got));
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
            .with_message(format!(
                "called with {} argument{} here",
                self.got,
                if self.got == 1 { "" } else { "s" }
            )),
        )
        .with_label(
            Label::new((
                defined.0.position().path().to_string(),
                defined.0.position().start().0..defined.0.position().end().0,
            ))
            .with_color(a)
            .with_message(format!(
                "defined here with {} argument{}",
                func.args().len(),
                if func.args().len() == 1 { "" } else { "s" }
            )),
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
