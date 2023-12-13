use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};

use crate::Error;
#[allow(unused)]
/// Unexpected token
pub struct IfInvalidOperator {
    /// The [`Token`] that was found
    tokens: Vec<Token>,
    /// The report
    report: Option<String>,
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

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.tokens
                .first()
                .unwrap()
                .position()
                .path()
                .as_str()
                .to_string(),
            &self
                .tokens
                .first()
                .unwrap()
                .position()
                .clone_with_end(*self.tokens.last().unwrap().position().end()),
        )]
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

impl IfInvalidOperator {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            report: None,
        }
        .report_generate()
    }

    pub fn code(tokens: Vec<Token>) -> Error {
        Error::Code(Box::new(Self::new(tokens)))
    }

    fn report_generate(mut self) -> Self {
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
            panic!("while reporting: {e}");
        }
        self.report = Some(String::from_utf8(out).unwrap_or_default());
        self
    }
}
