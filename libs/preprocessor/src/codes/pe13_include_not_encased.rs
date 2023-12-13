use std::sync::Arc;

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};

use crate::Error;

#[allow(unused)]
/// Unexpected token
pub struct IncludeNotEncased {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// The [`Symbol`] that the include is encased in
    encased_in: Option<Token>,
    /// The report
    report: Option<String>,
}

impl Code for IncludeNotEncased {
    fn ident(&self) -> &'static str {
        "PE13"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "include not encased".to_string()
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

impl IncludeNotEncased {
    pub fn new(token: Box<Token>, encased_in: Option<Token>) -> Self {
        Self {
            token,
            encased_in,
            report: None,
        }
        .report_generate()
    }

    pub fn code(token: Token, encased_in: Option<Token>) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), encased_in)))
    }

    fn report_generate(mut self) -> Self {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let start_token = self
            .encased_in
            .as_ref()
            .map_or(*self.token.clone(), Clone::clone);
        let span = start_token.position().start().0..self.token.position().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.position().path().as_str(), span.start..span.end))
                .with_color(a)
                .with_message(if self.encased_in.is_none() {
                    format!(
                        "try {}",
                        format!("<{}>", self.token.symbol().to_string().trim()).fg(a)
                    )
                } else {
                    format!(
                        "add {}",
                        self.encased_in
                            .as_ref()
                            .unwrap()
                            .symbol()
                            .matching_enclosure()
                            .unwrap()
                            .to_string()
                            .trim()
                            .fg(a)
                    )
                }),
        )
        .finish()
        .write_for_stdout(
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
