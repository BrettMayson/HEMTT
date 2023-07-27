use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_error::{tokens::Token, Code};
use lsp_types::{Diagnostic, Range};
use tracing::error;
use vfs::VfsPath;

#[allow(unused)]
/// Tried to use a [`Unit`](crate::context::Definition::Unit) as a function or value
pub struct ExpectedFunctionOrValue {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] of the function
    pub(crate) source: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
    /// Skipped tokens of Unit
    pub(crate) skipped: Vec<Token>,
    /// Likely a function
    pub(crate) likely_function: bool,
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
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.token.source().start().0..self.token.source().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.source().path_or_builtin(), span.start..span.end))
                .with_color(a)
                .with_message(if self.likely_function {
                    "tried to use as a function"
                } else {
                    "tried to use as a value"
                }),
        )
        .with_label(
            Label::new((
                self.source.source().path_or_builtin(),
                self.source.source().start().0..self.source.source().end().0,
            ))
            .with_color(a)
            .with_message("defined as a unit here"),
        )
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

    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.source().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.token.source().start().to_lsp(),
                end: self.token.source().end().to_lsp(),
            })
        ))
    }
}
