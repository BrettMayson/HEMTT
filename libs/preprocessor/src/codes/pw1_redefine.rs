use ariadne::{sources, ColorGenerator, Label, Report, ReportKind};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// Unexpected token
pub struct RedefineMacro {
    /// The [`Token`] that was defined
    pub(crate) token: Box<Token>,
    /// The original [`Token`] that was defined
    pub(crate) original: Box<Token>,
}

impl Code for RedefineMacro {
    fn ident(&self) -> &'static str {
        "PW1"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "redefining macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "redefining macro `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let color_token = colors.next();
        let color_original = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().start().0..self.token.position().end().0;
        if let Err(e) = Report::build(
            ReportKind::Warning,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.token.position().path().to_string(),
                span.start..span.end,
            ))
            .with_color(color_token)
            .with_message("redefining macro"),
        )
        .with_label(
            Label::new((
                self.original.position().path().to_string(),
                self.original.position().start().0..self.original.position().end().0,
            ))
            .with_color(color_original)
            .with_message("previous definition here"),
        )
        .with_help("`#undef` macros before redefining them")
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
                    self.original.position().path().to_string(),
                    self.original
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
}
