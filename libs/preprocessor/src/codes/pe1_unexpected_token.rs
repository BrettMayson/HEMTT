use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::error::{
    tokens::{Symbol, Token},
    Code,
};
use tracing::error;

#[allow(unused)]
/// Unexpected token
pub struct UnexpectedToken {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The valid [`Symbol`]s that were expected
    pub(crate) expected: Vec<Symbol>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for UnexpectedToken {
    fn ident(&self) -> &'static str {
        "PE1"
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
        let span = self.token.source().start().0..self.token.source().end().0;
        let report = Report::build(
            ReportKind::Error,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.source().path_or_builtin(), span.start..span.end))
                .with_color(a)
                .with_message("Unexpected token"),
        );
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
