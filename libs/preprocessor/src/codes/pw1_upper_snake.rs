use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use convert_case::{Case, Casing};
use hemtt_error::{tokens::Token, Code};
use tracing::error;

/// Unexpected token
pub struct UpperSnakeCase {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for UpperSnakeCase {
    fn ident(&self) -> &'static str {
        "PW1"
    }

    fn message(&self) -> String {
        "use upper snake case".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "use `{}`",
            self.token
                .symbol()
                .output()
                .replace('\n', "\\n")
                .to_case(Case::UpperSnake)
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
            ReportKind::Advice,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.source().path_or_builtin(), span.start..span.end))
                .with_color(a)
                .with_message(format!(
                    "try `{}`",
                    self.token
                        .symbol()
                        .output()
                        .trim()
                        .to_case(Case::UpperSnake)
                        .fg(a)
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
}
