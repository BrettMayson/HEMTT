use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// Unexpected token
pub struct IfInvalidOperator {
    /// The [`Token`] that was found
    pub(crate) tokens: Vec<Token>,
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
        let span = self.tokens.first().unwrap().position().start().0
            ..self.tokens.last().unwrap().position().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.tokens.first().unwrap().position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.tokens.first().unwrap().position().path().as_str(),
                span.start..span.end,
            ))
            .with_color(a)
            .with_message("invalid operator"),
        )
        .with_help(self.help().unwrap())
        .finish()
        .write_for_stdout(
            (
                self.tokens.first().unwrap().position().path().as_str(),
                Source::from(
                    self.tokens
                        .first()
                        .unwrap()
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
        let Some(path) = self.tokens.first().unwrap().position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.tokens.first().unwrap().position().start().to_lsp(),
                end: self.tokens.last().unwrap().position().end().to_lsp(),
            }),
        ))
    }
}
