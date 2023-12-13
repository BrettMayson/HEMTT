use std::sync::Arc;

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};

use crate::Error;

#[allow(unused)]
/// Tried to create a [`FunctionDefinition`](crate::context::FunctionDefinition) that has multi-token arguments
///
/// ```cpp
/// #define FUNC(my arg) ...
/// ```
pub struct DefineMissingComma {
    /// The [`Token`] of the previous arg
    previous: Box<Token>,
    /// The [`Token`] of the current arg
    current: Box<Token>,
    /// The report
    report: Option<String>,
}

impl Code for DefineMissingComma {
    fn ident(&self) -> &'static str {
        "PE5"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.current)
    }

    fn message(&self) -> String {
        "define arguments missing comma".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "define arguments missing comma `{}`",
            self.current.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.current.position().path().as_str().to_string(),
            self.current.position(),
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

impl DefineMissingComma {
    pub fn new(current: Box<Token>, previous: Box<Token>) -> Self {
        Self {
            current,
            previous,
            report: None,
        }
        .report_generate()
    }

    pub fn code(current: Token, previous: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(current), Box::new(previous))))
    }

    fn report_generate(mut self) -> Self {
        let mut colors = ColorGenerator::default();
        let color_comma = colors.next();
        let color_current = colors.next();
        let color_previous = colors.next();
        let mut out = Vec::new();
        let span = self.previous.position().start().0..self.current.position().end().0;
        let report = Report::build(
            ReportKind::Error,
            self.previous.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.current.position().path().as_str(),
                self.previous.position().start().0..self.previous.position().end().0,
            ))
            .with_color(color_previous),
        )
        .with_label(
            Label::new((
                self.current.position().path().as_str(),
                self.current.position().start().0..self.current.position().end().0,
            ))
            .with_color(color_current),
        )
        .with_label(
            Label::new((
                self.previous.position().path().as_str(),
                self.previous.position().start().0..self.current.position().end().0,
            ))
            .with_message(format!(
                "multiple tokens found without a {}",
                "comma".fg(color_comma)
            ))
            .with_color(color_comma),
        )
        .with_help(format!(
            "try `{}{} {}`",
            self.previous.to_string().fg(color_previous),
            ",".fg(color_comma),
            self.current.to_string().fg(color_current),
        ));
        if let Err(e) = report.finish().write_for_stdout(
            (
                self.current.position().path().as_str(),
                Source::from(
                    self.current
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
