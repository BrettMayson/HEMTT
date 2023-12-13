use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};

use crate::Error;

#[allow(unused)]
/// Unexpected token
pub struct UnexpectedToken {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// A vec of [`Token`]s that would be valid here
    expected: Vec<String>,
    /// The report
    report: Option<String>,
}

impl Code for UnexpectedToken {
    fn ident(&self) -> &'static str {
        "PE1"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "unexpected token".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "unexpected token `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
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
}

impl UnexpectedToken {
    pub fn new(token: Box<Token>, expected: Vec<String>) -> Self {
        Self {
            token,
            expected,
            report: None,
        }
        .report_generate()
    }

    pub fn code(token: Token, expected: Vec<String>) -> Error {
        Error::Code(Box::new(Self::new(Box::new(token), expected)))
    }

    fn report_generate(mut self) -> Self {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().span();
        let mut report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.position().path().as_str(), span.start..span.end))
                .with_color(a)
                .with_message("Unexpected token"),
        );
        if !self.expected.is_empty() {
            report = report.with_help(format!(
                "expected one of: {}",
                self.expected
                    .iter()
                    .map(|s| format!("`{}`", s.fg(a)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
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
