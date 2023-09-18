use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// Unexpected token
pub struct UnexpectedToken {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// A vec of [`Token`]s that would be valid here
    pub(crate) expected: Vec<String>,
}

impl Code for UnexpectedToken {
    fn ident(&self) -> &'static str {
        "PE1"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "unexpected token".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "unexpected token `{}`",
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
        let mut report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.position().path().as_str(), span.start..span.end))
                .with_color(a)
                .with_message("Unexpected token"),
        );
        if !self.expected.is_empty() {
            report = report.with_help(format!(
                "expected one of: {}",
                self.expected
                    .iter()
                    .map(|s| format!("`{}`", s.fg(a)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Err(e) = report.finish().write_for_stdout(
            (
                self.token.position().path().as_str(),
                Source::from(
                    self.token
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
            ),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }
}
