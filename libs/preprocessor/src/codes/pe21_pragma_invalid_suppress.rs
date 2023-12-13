use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::{
    reporting::{Annotation, AnnotationLevel, Code, Token},
    similar_values,
};

use crate::{processor::pragma::Suppress, Error};

#[allow(unused)]
/// An unknown `#pragma hemtt suppress` code
///
/// ```cpp
/// #pragma hemtt suppress unknown
/// ```
pub struct PragmaInvalidSuppress {
    /// The [`Token`] of the code
    token: Box<Token>,
    /// The report
    report: Option<String>,
}

impl Code for PragmaInvalidSuppress {
    fn ident(&self) -> &'static str {
        "PE21"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!(
            "unknown #pragma suppress `{}`",
            self.token.symbol().to_string(),
        )
    }

    fn help(&self) -> Option<String> {
        let similar = similar_values(self.token.to_string().as_str(), Suppress::as_slice());
        if similar.is_empty() {
            None
        } else {
            Some(format!(
                "Did you mean {}?",
                similar
                    .iter()
                    .map(|s| format!("`{s}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
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
                start: self.token.position().start().to_lsp() - 1,
                end: self.token.position().end().to_lsp(),
            }),
        ))
    }
}

impl PragmaInvalidSuppress {
    pub fn new(token: Box<Token>) -> Self {
        Self {
            token,
            report: None,
        }
        .report_generate()
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Box::new(Self::new(Box::new(token))))
    }

    fn report_generate(mut self) -> Self {
        let mut colors = ColorGenerator::default();
        let color_token = colors.next();
        let mut out = Vec::new();
        let mut report = Report::build(
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
            .with_message(format!(
                "unknown #pragma suppress `{}`",
                self.token.symbol().to_string().fg(color_token)
            )),
        );
        if let Some(help) = self.help() {
            report = report.with_help(help);
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
