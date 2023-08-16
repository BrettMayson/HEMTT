use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::error::{tokens::Token, Code};
use lsp_types::{Diagnostic, Range};
use tracing::error;
use vfs::VfsPath;

#[allow(unused)]
/// Unexpected token
pub struct IfInvalidOperator {
    /// The [`Token`] that was found
    pub(crate) tokens: Vec<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for IfInvalidOperator {
    fn ident(&self) -> &'static str {
        "PE15"
    }

    fn message(&self) -> String {
        "invalid #if operator".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "invalid #if operator `{}`",
            self.tokens
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<String>()
        )
    }

    fn help(&self) -> Option<String> {
        Some("valid operators are ==, !=, <, >, <=, >=".to_string())
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.tokens.first().unwrap().source().start().0
            ..self.tokens.last().unwrap().source().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.tokens.first().unwrap().source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.tokens.first().unwrap().source().path_or_builtin(),
                span.start..span.end,
            ))
            .with_color(a)
            .with_message("invalid operator"),
        )
        .with_help(self.help().unwrap())
        .finish()
        .write_for_stdout(
            (
                self.tokens.first().unwrap().source().path_or_builtin(),
                Source::from(
                    self.tokens
                        .first()
                        .unwrap()
                        .source()
                        .path()
                        .map_or_else(String::new, |path| {
                            path.read_to_string().unwrap_or_default()
                        }),
                ),
            ),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.tokens.first().unwrap().source().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.tokens.first().unwrap().source().start().to_lsp(),
                end: self.tokens.last().unwrap().source().end().to_lsp(),
            }),
        ))
    }
}
