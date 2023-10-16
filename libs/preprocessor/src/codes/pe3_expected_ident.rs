use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};
use tracing::error;

#[allow(unused)]
/// Expected an identifier, found something else
pub struct ExpectedIdent {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
}

impl Code for ExpectedIdent {
    fn ident(&self) -> &'static str {
        "PE3"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "expected identifier".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "expected identifier, found `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn report_generate(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().span();
        let report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.position().path().as_str(), span.start..span.end))
                .with_color(a)
                .with_message(format!(
                    "expected identifier, found `{}`",
                    self.token.symbol().to_string().replace('\n', "\\n")
                )),
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

    fn ci_generate(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.token.position().path().as_str().to_string(),
            self.token.position(),
        )]
    }

    #[cfg(feature = "lsp")]
    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.source().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.token.source().start().to_lsp(),
                end: self.token.source().end().to_lsp(),
            }),
        ))
    }
}
