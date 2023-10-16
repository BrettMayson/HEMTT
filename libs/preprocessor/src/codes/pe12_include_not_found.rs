use std::rc::Rc;

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};
use tracing::error;

/// An include was not found
pub struct IncludeNotFound {
    /// The target that was not found
    pub token: Vec<Token>,
}

impl IncludeNotFound {
    pub fn new(token: Vec<Rc<Token>>) -> Self {
        Self {
            token: token.into_iter().map(|t| t.as_ref().clone()).collect(),
        }
    }
}

impl Code for IncludeNotFound {
    fn ident(&self) -> &'static str {
        "PE12"
    }

    fn message(&self) -> String {
        "include not found".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "include not found `{}`",
            self.token
                .iter()
                .map(|t| t.symbol().to_string())
                .collect::<String>()
                .replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn report_generate(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.token.first().unwrap().position().start().0
            ..self.token.last().unwrap().position().end().0;
        let token = self.token.first().unwrap();
        if let Err(e) = Report::build(
            ReportKind::Error,
            token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((token.position().path().as_str(), span.start..span.end))
                .with_color(colors.next()),
        )
        .finish()
        .write_for_stdout(
            (
                token.position().path().as_str(),
                Source::from(token.position().path().read_to_string().unwrap_or_default()),
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
            self.token
                .first()
                .unwrap()
                .position()
                .path()
                .as_str()
                .to_string(),
            &self
                .token
                .first()
                .unwrap()
                .position()
                .clone_with_end(*self.token.last().unwrap().position().end()),
        )]
    }

    #[cfg(feature = "lsp")]
    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.first().unwrap().position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range::new(
                self.token.first().unwrap().position().start().to_lsp(),
                self.token.last().unwrap().position().end().to_lsp(),
            )),
        ))
    }
}
