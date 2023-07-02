use ariadne::{Label, Report, ReportKind, Source};
use hemtt_error::{tokens::Token, Code};
use tracing::error;

#[allow(unused)]
/// Tried to use a [`Unit`](crate::context::Definition::Unit) as a function or value
pub struct ExpectedFunctionOrValue {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
    /// Skipped tokens of Unit
    pub(crate) skipped: Vec<Token>,
}

impl Code for ExpectedFunctionOrValue {
    fn ident(&self) -> &'static str {
        "PE11"
    }

    fn message(&self) -> String {
        "expected function or value, found unit".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "expected function or value, found unit `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "expected function or value, found unit `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        ))
    }

    fn generate_report(&self) -> Option<String> {
        let mut out = Vec::new();
        let span = self.token.source().start().0..self.token.source().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(Label::new((
            self.token.source().path_or_builtin(),
            span.start..span.end,
        )))
        .finish()
        .write_for_stdout(
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
