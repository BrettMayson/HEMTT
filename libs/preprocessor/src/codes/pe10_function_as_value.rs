use ariadne::{sources, ColorGenerator, Label, Report, ReportKind};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// Tried to use a [`FunctionDefinition`](crate::context::FunctionDefinition) as a value
pub struct FunctionAsValue {
    /// The [`Token`] that was called
    pub(crate) token: Box<Token>,
    /// The [`Token`] of the function
    pub(crate) source: Box<Token>,
}

impl Code for FunctionAsValue {
    fn ident(&self) -> &'static str {
        "PE10"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "attempted to use a function as a value".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "attempted to use a function as a value `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
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
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.token.position().path().to_string(),
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
            .with_message("expecting arguments"),
        )
        .with_label(
            Label::new((
                self.source.position().path().to_string(),
                self.source.position().start().0..self.source.position().end().0,
            ))
            .with_color(a)
            .with_message("defined as a function here"),
        )
        .finish()
        .write_for_stdout(
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
                    self.source.position().path().to_string(),
                    self.source
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
    fn generate_lsp(&self) -> Option<(vfs::VfsPath, Diagnostic)> {
        let Some(path) = self.from.position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.from.position().start().to_lsp(),
                end: self.from.position().end().to_lsp(),
            }),
        ))
    }
}
