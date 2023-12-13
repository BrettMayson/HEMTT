use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::{
    reporting::{Annotation, AnnotationLevel, Code, Token},
    similar_values,
};

use crate::Error;

#[allow(unused)]
/// An unknown `#pragma` directive
///
/// ```cpp
/// #pragma hemtt unknown
/// ```
pub struct PragmaInvalidScope {
    /// The [`Token`] of the scope
    token: Box<Token>,
    /// Are we in the root config?
    root: bool,
    /// The report
    report: Option<String>,
}

impl Code for PragmaInvalidScope {
    fn ident(&self) -> &'static str {
        "PE20"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!(
            "unknown #pragma scope `{}`",
            self.token.symbol().to_string(),
        )
    }

    fn label_message(&self) -> String {
        format!(
            "unknown #pragma scope `{}`",
            self.token.symbol().to_string()
        )
    }

    fn help(&self) -> Option<String> {
        let scopes = if self.root {
            vec!["line", "file", "config"]
        } else {
            vec!["line", "file"]
        };
        let did_you_mean = {
            let similar = similar_values(self.token.to_string().as_str(), &scopes);
            if similar.is_empty() {
                String::new()
            } else {
                format!(
                    ", did you mean {}?",
                    similar
                        .iter()
                        .map(|s| format!("`{s}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        };
        let scopes = scopes
            .iter()
            .map(|s| format!("`{s}`"))
            .collect::<Vec<_>>()
            .join(", ");
        Some(format!("Valid scopes here: {scopes}{did_you_mean}"))
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

impl PragmaInvalidScope {
    pub fn new(token: Box<Token>, root: bool) -> Self {
        Self {
            token,
            root,
            report: None,
        }
        .report_generate()
    }

    pub fn code(token: Token, root: bool) -> Error {
        Error::Code(Box::new(Self::new(Box::new(token), root)))
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
                "unknown #pragma scope `{}`",
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
