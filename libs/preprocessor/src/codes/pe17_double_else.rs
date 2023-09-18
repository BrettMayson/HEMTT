use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Code, Token};
use tracing::error;

#[allow(unused)]
/// An `#else` [`IfState`] was found after another `#else`
///
/// ```cpp
/// #if 1
/// #else
/// #else
/// #endif
/// ```
pub struct DoubleElse {
    /// The [`Token`] of the new `#else`
    pub(crate) token: Box<Token>,
    /// The [`Token`] of the previous `#else`
    pub(crate) previous: Box<Token>,
    /// The [`Token`] of the `#if` that this `#else` is in
    pub(crate) if_token: Box<Token>,
}

impl Code for DoubleElse {
    fn ident(&self) -> &'static str {
        "PE17"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "multiple `#else` directives found in a single `#if`".to_string()
    }

    fn label_message(&self) -> String {
        "second `#else` directive".to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let color_token = colors.next();
        let color_previous = colors.next();
        let color_if = colors.next();
        let mut out = Vec::new();
        let report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            self.token.position().start().offset(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.token.position().path().as_str(),
                self.token.position().start().offset()..self.token.position().end().offset(),
            ))
            .with_color(color_token)
            .with_message(format!("second `{}` directive", "#else".fg(color_token))),
        )
        .with_label(
            Label::new((
                self.previous.position().path().as_str(),
                self.previous.position().start().offset()..self.previous.position().end().offset(),
            ))
            .with_color(color_previous)
            .with_message(format!("first `{}` directive", "#else".fg(color_previous))),
        )
        .with_label(
            Label::new((
                self.if_token.position().path().as_str(),
                self.if_token.position().start().offset()..self.if_token.position().end().offset(),
            ))
            .with_color(color_if)
            .with_message(format!("`{}` directive", "#if".fg(color_if))),
        );
        if let Err(e) = report.finish().write_for_stdout(
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
