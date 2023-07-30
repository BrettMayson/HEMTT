use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_error::{tokens::Token, Code};
use lsp_types::{Diagnostic, Range};
use tracing::error;
use vfs::VfsPath;

#[allow(unused)]
/// Tried to change a built-in macro
pub struct ChangeBuiltin {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for ChangeBuiltin {
    fn ident(&self) -> &'static str {
        "PE6"
    }

    fn message(&self) -> String {
        "tried changing a built-in macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "built-in macro `{}`",
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
                .with_message(format!(
                    "built-in macro `{}`",
                    self.token.symbol().output().trim().fg(a)
                )),
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
            }),
        ))
    }
}
