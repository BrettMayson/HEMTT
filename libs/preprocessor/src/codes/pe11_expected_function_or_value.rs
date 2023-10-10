use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// Tried to use a [`Unit`](crate::context::Definition::Unit) as a function or value
pub struct ExpectedFunctionOrValue {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] of the function
    pub(crate) source: Box<Token>,
    /// Likely a function
    pub(crate) likely_function: bool,
}

impl Code for ExpectedFunctionOrValue {
    fn ident(&self) -> &'static str {
        "PE11"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "expected function or value, found unit".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "expected function or value, found unit `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "expected function or value, found unit `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        ))
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().start().0..self.token.position().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.position().path().as_str(), span.start..span.end))
                .with_color(a)
                .with_message(if self.likely_function {
                    "tried to use as a function"
                } else {
                    "tried to use as a value"
                }),
        )
        .with_label(
            Label::new((
                self.source.position().path().as_str(),
                self.source.position().start().0..self.source.position().end().0,
            ))
            .with_color(a)
            .with_message("defined as a unit here"),
        )
        .finish()
        .write_for_stdout(
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
