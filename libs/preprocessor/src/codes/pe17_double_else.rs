use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};

use crate::Error;

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
    token: Box<Token>,
    /// The [`Token`] of the previous `#else`
    previous: Box<Token>,
    /// The [`Token`] of the `#if` that this `#else` is in
    if_token: Box<Token>,
    /// The report
    report: Option<String>,
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

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.token.position().path().as_str().to_string(),
            self.token.position(),
        )]
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

impl DoubleElse {
    pub fn new(token: Box<Token>, previous: Box<Token>, if_token: Box<Token>) -> Self {
        Self {
            token,
            previous,
            if_token,
            report: None,
        }
        .report_generate()
    }

    pub fn code(token: Token, previous: Token, if_token: Token) -> Error {
        Error::Code(Box::new(Self::new(
            Box::new(token),
            Box::new(previous),
            Box::new(if_token),
        )))
    }

    fn report_generate(mut self) -> Self {
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
            panic!("while reporting: {e}");
        }
        self.report = Some(String::from_utf8(out).unwrap_or_default());
        self
    }
}
