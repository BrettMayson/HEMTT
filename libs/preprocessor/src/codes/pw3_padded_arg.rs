use ariadne::{sources, ColorGenerator, Label, Report, ReportKind};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};
use tracing::error;

#[allow(unused)]
/// Unexpected token
pub struct PaddedArg {
    /// The [`Token`] that was found to be padding an arg
    pub(crate) token: Box<Token>,
}

impl Code for PaddedArg {
    fn ident(&self) -> &'static str {
        "PW3"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "padding a macro argument".to_string()
    }

    fn label_message(&self) -> String {
        "padding a macro argument is likely unintended".to_string()
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn report_generate(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let color_token = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().span();
        if let Err(e) = Report::build(
            ReportKind::Warning,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.token.position().path().to_string(),
                span.start..span.end,
            ))
            .with_color(color_token)
            .with_message("padding a macro argument is likely unintended"),
        )
        .finish()
        .write_for_stdout(
            sources(vec![(
                self.token.position().path().to_string(),
                self.token
                    .position()
                    .path()
                    .read_to_string()
                    .unwrap_or_default(),
            )]),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    fn ci_generate(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Warning,
            self.token.position().path().as_str().to_string(),
            self.token.position(),
        )]
    }
}
