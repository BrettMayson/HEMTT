use ariadne::{sources, ColorGenerator, Label, Report, ReportKind};
use hemtt_error::{tokens::Token, Code};
use lsp_types::{Diagnostic, Range};
use tracing::error;

#[allow(unused)]
/// Tried to use a [`FunctionDefinition`](crate::context::FunctionDefinition) as a value
pub struct FunctionAsValue {
    /// The [`Token`] that was found instead of `(`
    pub(crate) token: Box<Token>,
    /// The [`Token`] of the function
    pub(crate) source: Box<Token>,
    /// The [`Token`] that called the definition
    pub(crate) from: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for FunctionAsValue {
    fn ident(&self) -> &'static str {
        "PE10"
    }

    fn message(&self) -> String {
        "attempted to use a function as a value".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "attempted to use a function as a value `{}`",
            self.from.symbol().output().replace('\n', "\\n")
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
                .with_message("expecting argument list here"),
        )
        .with_label(
            Label::new((
                self.source.source().path_or_builtin(),
                self.source.source().start().0..self.source.source().end().0,
            ))
            .with_color(a)
            .with_message("defined as a function here"),
        )
        .finish()
        .write_for_stdout(
            sources(vec![
                (
                    self.token.source().path_or_builtin(),
                    self.token.source().path().map_or_else(String::new, |path| {
                        path.read_to_string().unwrap_or_default()
                    }),
                ),
                (
                    self.source.source().path_or_builtin(),
                    self.source
                        .source()
                        .path()
                        .map_or_else(String::new, |path| {
                            path.read_to_string().unwrap_or_default()
                        }),
                ),
            ]),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    fn generate_lsp(&self) -> Option<(vfs::VfsPath, Diagnostic)> {
        let Some(path) = self.from.source().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.from.source().start().to_lsp(),
                end: self.from.source().end().to_lsp(),
            })
        ))
    }
}
